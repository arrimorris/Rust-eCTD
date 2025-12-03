use std::process::Command;
use std::path::Path;
use std::env;

#[test]
fn test_full_lifecycle() {
    // 1. Setup paths
    // "CARGO_MANIFEST_DIR" now points to crates/ectd_cli
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let cli_root = Path::new(manifest_dir);

    // We need to go up two levels to reach the workspace root (ectd_v4_forge)
    let workspace_root = cli_root
        .parent().expect("No parent")
        .parent().expect("No grandparent");

    let sample_xml = workspace_root.join("sample_submission.xml");

    // We put the output in the workspace target directory, not the crate one, to keep it clean
    let export_dir = workspace_root.join("target/test_export");

    // Ensure we have a dummy PDF to ingest (referenced in sample_submission.xml)
    // This is relative to where the 'ingest' command runs (workspace_root)
    let pdf_dir = workspace_root.join("m1/us");
    std::fs::create_dir_all(&pdf_dir).expect("Failed to create PDF dir");
    std::fs::write(pdf_dir.join("cover.pdf"), "DUMMY PDF CONTENT").expect("Failed to create dummy PDF");

    // 2. Run Ingest
    println!("🧪 Running Ingest...");
    // Note: We set current_dir to workspace_root so the relative paths in XML work
    let ingest_output = Command::new("cargo")
        .args(&["run", "-p", "ectd_cli", "--", "ingest", "--file"])
        .arg(&sample_xml)
        .current_dir(workspace_root)
        .env("DATABASE_URL", "postgres://ectd_admin:secure_password_123@localhost:5432/ectd_v4")
        .env("S3_ENDPOINT", "http://localhost:9000")
        .env("AWS_ACCESS_KEY_ID", "minio_admin")
        .env("AWS_SECRET_ACCESS_KEY", "secure_minio_123")
        .env("AWS_REGION", "us-east-1")
        // We explicitly set the S3 bucket env var because config.rs requires it or defaults
        .env("S3_BUCKET", "ectd-documents")
        .output()
        .expect("Failed to run ingest");

    if !ingest_output.status.success() {
        eprintln!("Ingest Stderr: {}", String::from_utf8_lossy(&ingest_output.stderr));
        panic!("Ingest failed");
    }

    // Extract UUID from stdout
    let stdout = String::from_utf8_lossy(&ingest_output.stdout);
    println!("Ingest Output: {}", stdout);

    let uuid_line = stdout.lines().find(|l| l.contains("Primary Key (UUID):")).expect("UUID not found in output");
    let uuid = uuid_line.split(": ").nth(1).unwrap().trim();
    println!("   🔑 Captured UUID: {}", uuid);

    // 3. Run Export
    println!("🧪 Running Export...");
    let export_output = Command::new("cargo")
        .args(&["run", "-p", "ectd_cli", "--", "export", "--id", uuid, "--output"])
        .arg(&export_dir)
        .current_dir(workspace_root)
        .env("DATABASE_URL", "postgres://ectd_admin:secure_password_123@localhost:5432/ectd_v4")
        .env("S3_ENDPOINT", "http://localhost:9000")
        .env("AWS_ACCESS_KEY_ID", "minio_admin")
        .env("AWS_SECRET_ACCESS_KEY", "secure_minio_123")
        .env("AWS_REGION", "us-east-1")
        .env("S3_BUCKET", "ectd-documents")
        .output()
        .expect("Failed to run export");

    if !export_output.status.success() {
        eprintln!("Export Stderr: {}", String::from_utf8_lossy(&export_output.stderr));
        panic!("Export failed");
    }

    // 4. Verify Artifacts
    assert!(export_dir.join("submissionunit.xml").exists(), "XML missing");
    assert!(export_dir.join("sha256.txt").exists(), "Manifest missing");
    assert!(export_dir.join("m1/us/cover.pdf").exists(), "Physical file missing");

    println!("✅ End-to-End Test Passed!");
}
