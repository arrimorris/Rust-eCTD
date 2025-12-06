use crate::EctdService;
use anyhow::{Context, Result};
use uuid::Uuid;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Write, BufWriter};
use futures::stream::{self, StreamExt};
use tokio::io::AsyncWriteExt;
use sha2::{Sha256, Digest};
use ectd_db::repository::SubmissionRepository;

impl EctdService {
    pub async fn export_submission(&self, id: Uuid, output_dir: PathBuf) -> Result<PathBuf> {
        // 1. Fetch Data
        let repo = SubmissionRepository::new(self.pool.clone());
        let unit = repo.get_submission(id).await
            .context("Failed to fetch submission from DB")?;

        // 2. Prepare Output Directory
        if output_dir.exists() {
            // Optional: Clean dir or warn. For now, we overwrite.
        }
        fs::create_dir_all(&output_dir)?;

        // 3. Download Files (Parallel)
        let downloads = stream::iter(unit.documents.iter())
            .map(|doc| {
                let client = self.s3.clone();
                let bucket = self.bucket.clone();
                let out_dir = output_dir.clone();
                let doc_id = doc.id.clone();
                // Ensure text and reference are available
                let ref_path = doc.text.reference.value.clone();

                async move {
                    let rel_path = Path::new(&ref_path);
                    let full_out_path = out_dir.join(rel_path);

                    if let Some(parent) = full_out_path.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    let mut stream = client.get_object()
                        .bucket(bucket)
                        .key(&doc_id.to_string()) // Convert UUID to string key
                        .send()
                        .await?
                        .body;

                    let mut file = tokio::fs::File::create(&full_out_path).await?;
                    while let Some(bytes) = stream.try_next().await? {
                        file.write_all(&bytes).await?;
                    }

                    Ok::<PathBuf, anyhow::Error>(full_out_path)
                }
            })
            .buffer_unordered(10); // 10 concurrent downloads

        let results: Vec<_> = downloads.collect().await;

        // Check for download errors
        for res in &results {
            if let Err(e) = res {
                return Err(anyhow::anyhow!("File download failed: {}", e));
            }
        }

        // 4. Build Manifest (sha256.txt)
        let mut manifest_entries = Vec::new();
        for res in results {
            let path = res?;
            let rel_str = path.strip_prefix(&output_dir)?.to_string_lossy().replace("\\", "/");
            let hash = calculate_file_hash(&path)?;
            manifest_entries.push((hash, rel_str));
        }

        // 5. Generate XML
        let final_xml = unit.to_xml()?;
        let xml_path = output_dir.join("submissionunit.xml");
        fs::write(&xml_path, final_xml)?;

        let xml_hash = calculate_file_hash(&xml_path)?;
        manifest_entries.push((xml_hash, "submissionunit.xml".to_string()));

        // 6. Write Manifest
        let manifest_path = output_dir.join("sha256.txt");
        let mut manifest_file = BufWriter::new(File::create(manifest_path)?);
        for (hash, filename) in manifest_entries {
            writeln!(manifest_file, "{}  {}", hash, filename)?;
        }

        Ok(output_dir)
    }
}

// Synchronous Hashing Helper
fn calculate_file_hash(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    Ok(hex::encode(hasher.finalize()))
}
