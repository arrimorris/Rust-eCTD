use crate::EctdService;
use anyhow::{Context, Result};
use uuid::Uuid;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Write, BufWriter};
use futures::stream::{self, Stream, StreamExt};
use tokio::io::AsyncWriteExt;
use sha2::{Sha256, Digest};
use ectd_db::repository::SubmissionRepository;
use async_stream::stream;
use serde::{Serialize, Deserialize};
use std::pin::Pin;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportProgress {
    pub file_name: String,      // "m1/us/cover.pdf"
    pub processed_files: usize, // 45
    pub total_files: usize,     // 100
    pub bytes_processed: u64,   // For a detailed bar
    pub status: String,         // "Downloading", "Hashing", "Complete"
}

impl EctdService {
    pub fn export_submission_stream(
        &self,
        id: Uuid,
        output_dir: PathBuf,
    ) -> Pin<Box<dyn Stream<Item = Result<ExportProgress, anyhow::Error>> + Send + '_>> {
        Box::pin(stream! {
            // 1. Fetch Data
            let repo = SubmissionRepository::new(self.pool.clone());
            let unit = match repo.get_submission(id).await.context("Failed to fetch submission from DB") {
                Ok(u) => u,
                Err(e) => {
                    yield Err(e);
                    return;
                }
            };

            let total_docs = unit.documents.len();
            let mut processed_count = 0;

            // 2. Prepare Output Directory
            if let Err(e) = fs::create_dir_all(&output_dir) {
                yield Err(anyhow::anyhow!("Failed to create output dir: {}", e));
                return;
            }

            yield Ok(ExportProgress {
                file_name: "Starting Export...".to_string(),
                processed_files: 0,
                total_files: total_docs,
                bytes_processed: 0,
                status: "Initializing".to_string(),
            });

            // 3. Download Files (Parallel)
            // Use clone of documents to own the data in the stream, avoiding complex lifetime issues with async closures
            let docs_to_download = unit.documents.clone();
            let mut downloads = stream::iter(docs_to_download.into_iter())
                .map(|doc| {
                    let client = self.s3.clone();
                    let bucket = self.bucket.clone();
                    let out_dir = output_dir.clone();
                    let doc_id = doc.id;
                    let ref_path = doc.text.reference.value;

                    async move {
                        let rel_path = Path::new(&ref_path);
                        let full_out_path = out_dir.join(rel_path);

                        if let Some(parent) = full_out_path.parent() {
                             let _ = fs::create_dir_all(parent);
                        }

                        let mut stream = client.get_object()
                            .bucket(bucket)
                            .key(&doc_id.to_string())
                            .send()
                            .await?
                            .body;

                        let mut file = tokio::fs::File::create(&full_out_path).await?;
                        let mut bytes_written = 0;
                        while let Some(bytes) = stream.try_next().await? {
                            file.write_all(&bytes).await?;
                            bytes_written += bytes.len() as u64;
                        }

                        Ok::<(PathBuf, String, u64), anyhow::Error>((full_out_path, ref_path, bytes_written))
                    }
                })
                .buffer_unordered(10);

            let mut manifest_entries = Vec::new();
            let mut total_bytes = 0;

            while let Some(result) = downloads.next().await {
                match result {
                    Ok((path, ref_path, bytes)) => {
                        processed_count += 1;
                        total_bytes += bytes;

                        let hash = calculate_file_hash(&path);
                        let hash = match hash {
                            Ok(h) => h,
                            Err(e) => {
                                 yield Err(anyhow::anyhow!("Hashing failed: {}", e));
                                 return;
                            }
                        };

                        let rel_str = match path.strip_prefix(&output_dir) {
                            Ok(p) => p.to_string_lossy().replace("\\", "/"),
                            Err(e) => {
                                yield Err(anyhow::anyhow!("Path strip prefix failed: {}", e));
                                return;
                            }
                        };
                        manifest_entries.push((hash, rel_str));

                        yield Ok(ExportProgress {
                            file_name: ref_path,
                            processed_files: processed_count,
                            total_files: total_docs,
                            bytes_processed: total_bytes,
                            status: "Downloading".to_string(),
                        });
                    },
                    Err(e) => {
                        yield Err(anyhow::anyhow!("File download failed: {}", e));
                        return;
                    }
                }
            }

            // 5. Generate XML
            yield Ok(ExportProgress {
                file_name: "submissionunit.xml".to_string(),
                processed_files: processed_count,
                total_files: total_docs,
                bytes_processed: total_bytes,
                status: "Generating XML".to_string(),
            });

            let final_xml = match unit.to_xml() {
                Ok(x) => x,
                Err(e) => {
                    yield Err(anyhow::anyhow!("XML generation failed: {}", e));
                    return;
                }
            };

            let xml_path = output_dir.join("submissionunit.xml");
            if let Err(e) = fs::write(&xml_path, final_xml) {
                 yield Err(anyhow::anyhow!("Failed to write XML: {}", e));
                 return;
            }

            let xml_hash = match calculate_file_hash(&xml_path) {
                Ok(h) => h,
                Err(e) => {
                    yield Err(anyhow::anyhow!("XML hashing failed: {}", e));
                    return;
                }
            };
            manifest_entries.push((xml_hash, "submissionunit.xml".to_string()));

            // 6. Write Manifest
            yield Ok(ExportProgress {
                file_name: "sha256.txt".to_string(),
                processed_files: processed_count,
                total_files: total_docs,
                bytes_processed: total_bytes,
                status: "Finalizing Manifest".to_string(),
            });

            let manifest_path = output_dir.join("sha256.txt");
            let manifest_file = match File::create(manifest_path) {
                Ok(f) => f,
                Err(e) => {
                    yield Err(anyhow::anyhow!("Failed to create manifest file: {}", e));
                    return;
                }
            };
            let mut manifest_file = BufWriter::new(manifest_file);
            for (hash, filename) in manifest_entries {
                let line = format!("{}  {}\n", hash, filename);
                if let Err(e) = manifest_file.write_all(line.as_bytes()) {
                    yield Err(anyhow::anyhow!("Failed to write to manifest: {}", e));
                    return;
                }
            }

            yield Ok(ExportProgress {
                file_name: "Done".to_string(),
                processed_files: total_docs,
                total_files: total_docs,
                bytes_processed: total_bytes,
                status: "Complete".to_string(),
            });
        })
    }
}

// Synchronous Hashing Helper
fn calculate_file_hash(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    Ok(hex::encode(hasher.finalize()))
}
