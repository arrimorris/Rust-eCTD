pub mod documents;

use sqlx::PgPool;
use aws_sdk_s3::Client as S3Client;

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
}
