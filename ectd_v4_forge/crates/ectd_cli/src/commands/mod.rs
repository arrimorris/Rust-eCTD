pub mod ingest;
pub mod import_standard;
pub mod validate;
pub mod rebuild;
pub mod init;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Ingest a raw dataset and format it for eCTD
    Ingest(ingest::IngestArgs),
    
    /// Import CDISC standards from CSV
    ImportStandard(import_standard::ImportStandardArgs),
}
pub mod forge_data;
pub mod export;
pub mod add_doc;
