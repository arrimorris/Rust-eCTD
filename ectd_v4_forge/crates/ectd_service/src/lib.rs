pub mod documents;
pub mod submission;
pub mod export;

use sqlx::PgPool;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::error::SdkError;

// Re-export common types
pub use documents::AddDocumentParams;
pub use submission::InitSubmissionParams;
pub use export::ExportProgress;

#[derive(Clone)]
pub struct EctdService {
    pub pool: PgPool,
    pub s3: S3Client,
    pub bucket: String,
}

impl EctdService {
    pub fn new(pool: PgPool, s3: S3Client, bucket: String) -> Self {
        Self {
            pool,
            s3,
            bucket,
        }
    }

    /// Idempotent check: Creates the bucket if it doesn't exist.
    /// Returns Ok if ready, Err if critical failure.
    pub async fn ensure_bucket(&self) -> Result<(), anyhow::Error> {
        // 1. Check existence (HeadBucket)
        let head = self.s3.head_bucket()
            .bucket(&self.bucket)
            .send()
            .await;

        match head {
            Ok(_) => Ok(()), // Exists
            Err(SdkError::ServiceError(err)) if err.err().is_not_found() => {
                // 2. Create if missing (404)
                println!("⚠️  Bucket '{}' not found. Creating...", self.bucket);
                self.s3.create_bucket()
                    .bucket(&self.bucket)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to create bucket: {}", e))?;

                println!("✅ Bucket created.");
                Ok(())
            },
            Err(e) => Err(anyhow::anyhow!("Failed to check bucket state: {}", e)),
        }
    }
}
