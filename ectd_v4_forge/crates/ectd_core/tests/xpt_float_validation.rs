// crates/ectd_core/tests/xpt_float_validation.rs
//
// CRITICAL TEST: Validates that XPT v5 IBM 370 float conversion preserves precision
// for clinical trial data submission to FDA

use std::fs::File;
use std::io::{BufWriter, Cursor, Read, Seek, SeekFrom};
use ectd_core::sdtm::xpt_v5::XptWriter;

/// Test cases covering critical clinical data scenarios
struct FloatTestCase {
    name: &'static str,
    value: f64,
    tolerance: f64, // Acceptable error margin
    context: &'static str,
}

const TEST_CASES: &[FloatTestCase] = &[
    FloatTestCase {
        name: "Exact_Integer",
        value: 100.0,
        tolerance: 0.0,
        context: "Patient count should be exact",
    },
    FloatTestCase {
        name: "Dosage_Precision",
        value: 150.5,
        tolerance: 0.001, // 1mg precision for 150mg dose
        context: "Drug dosage in mg",
    },
    FloatTestCase {
        name: "Lab_Value_High_Precision",
        value: 7.234,
        tolerance: 0.0001, // 0.1 mmol/L precision
        context: "Blood glucose in mmol/L",
    },
    FloatTestCase {
        name: "Very_Small_Value",
        value: 0.0001,
        tolerance: 0.00001,
        context: "Trace elements or p-values",
    },
    FloatTestCase {
        name: "Large_Value",
        value: 999999.99,
        tolerance: 0.01,
        context: "Cost or large measurements",
    },
    FloatTestCase {
        name: "Negative_Value",
        value: -37.5,
        tolerance: 0.01,
        context: "Temperature in Celsius",
    },
    FloatTestCase {
        name: "Time_Half_Hour",
        value: 24.5,
        tolerance: 0.01,
        context: "Time in hours (24.5 = 24h 30m)",
    },
    FloatTestCase {
        name: "Statistical_Edge_Case",
        value: 0.05,
        tolerance: 0.0001,
        context: "p-value threshold",
    },
    FloatTestCase {
        name: "Zero",
        value: 0.0,
        tolerance: 0.0,
        context: "Baseline or null measurements",
    },
];

#[test]
fn test_xpt_float_round_trip() {
    println!("\n=== XPT Float Conversion Validation ===\n");

    let mut failures = Vec::new();

    for test in TEST_CASES {
        // Create temporary XPT file in memory
        let mut buffer = Cursor::new(Vec::new());

        // Write the test value
        {
            let writer = BufWriter::new(&mut buffer);
            let mut xpt = XptWriter::new(writer, "TEST");

            // Define a single numeric variable
            xpt.write_header(&[("VALUE", "Num")]).expect("Header write failed");

            // Write the test value as a row
            xpt.write_row(&[test.value.to_string()]).expect("Row write failed");
        }

        // Read back the binary representation
        buffer.seek(SeekFrom::Start(0)).expect("Seek failed");
        let bytes = buffer.into_inner();

        // Parse the float value from the XPT binary format
        // XPT structure: Headers (~560 bytes) + Data
        // We need to extract the 8-byte IBM 370 float from the data section

        let result = extract_and_decode_first_value(&bytes);

        match result {
            Ok(decoded_value) => {
                let error = (decoded_value - test.value).abs();
                let within_tolerance = error <= test.tolerance;

                if within_tolerance {
                    println!("✅ {}: {} → {} (error: {:.10})",
                        test.name, test.value, decoded_value, error);
                } else {
                    println!("❌ {}: {} → {} (error: {:.10}, tolerance: {:.10})",
                        test.name, test.value, decoded_value, error, test.tolerance);
                    failures.push((test, decoded_value, error));
                }
            }
            Err(e) => {
                println!("❌ {}: Failed to decode - {}", test.name, e);
                failures.push((test, 0.0, f64::INFINITY));
            }
        }
    }

    // Report results
    println!("\n=== Summary ===");
    println!("Passed: {}/{}", TEST_CASES.len() - failures.len(), TEST_CASES.len());

    if !failures.is_empty() {
        println!("\n=== FAILURES ===");
        for (test, decoded, error) in &failures {
            println!("❌ {} ({})", test.name, test.context);
            println!("   Expected: {} (±{})", test.value, test.tolerance);
            println!("   Got:      {} (error: {})", decoded, error);
        }

        panic!("\n🚨 XPT FLOAT CONVERSION FAILED\n\
                This will corrupt clinical trial data submitted to FDA.\n\
                {} test(s) exceeded acceptable precision loss.\n\
                \n\
                RECOMMENDED ACTION:\n\
                1. Implement proper IBM 370 conversion using 'ibm_float' crate\n\
                2. Or validate that current implementation meets your precision requirements\n\
                3. Document acceptable precision loss in clinical context\n",
                failures.len());
    }

    println!("\n✅ All float conversions within acceptable tolerance");
}

/// Extracts and decodes the first numeric value from XPT binary data
/// This version expects 8-byte IBM 370 float data.
fn extract_and_decode_first_value(bytes: &[u8]) -> Result<f64, String> {
    // XPT file structure (simplified):
    // - Multiple 80-byte header records
    // - NAMESTR records (140 bytes per variable)
    // - OBS header record
    // - Data (continuous stream of values)

    // Find the start of data by locating "OBS     HEADER RECORD"
    let obs_marker = b"OBS     HEADER RECORD";
    let data_start = bytes.windows(obs_marker.len())
        .position(|window| window == obs_marker)
        .ok_or("Could not find OBS header marker")?;

    // Data starts after the OBS header record (80 bytes)
    let data_offset = data_start + 80;

    // We expect exactly 8 bytes for a Num variable
    if data_offset + 8 > bytes.len() {
        return Err("File too short to contain data".to_string());
    }

    let value_bytes = &bytes[data_offset..data_offset + 8];

    // Convert IBM 370 bytes to f64
    Ok(ibm370_to_f64(value_bytes))
}

/// Helper to decode IBM 370 double precision float
fn ibm370_to_f64(bytes: &[u8]) -> f64 {
    if bytes.iter().all(|&b| b == 0) {
        return 0.0;
    }

    let sign = (bytes[0] & 0x80) != 0;
    let exponent = (bytes[0] & 0x7F) as i32 - 64;

    let mut fraction: u64 = 0;
    for i in 1..8 {
        fraction = (fraction << 8) | bytes[i] as u64;
    }

    // IBM fraction is 0.xxxxxx... (base 16)
    // Value = 16^exponent * (fraction / 2^56)

    let f_val = fraction as f64 / (1u64 << 56) as f64;
    let val = f_val * 16.0f64.powi(exponent);

    if sign { -val } else { val }
}

/// EXTERNAL VALIDATION TEST
/// This test generates CSV and XPT files for manual verification with external tools
#[test]
#[ignore] // Run with: cargo test --test xpt_float_validation -- --ignored
fn generate_validation_files_for_external_check() {
    use std::io::Write;

    // Create CSV with test data
    let csv_path = "target/test_float_validation.csv";
    let mut csv = File::create(csv_path).expect("Failed to create CSV");
    writeln!(csv, "TEST_NAME,VALUE,CONTEXT").unwrap();

    for test in TEST_CASES {
        writeln!(csv, "{},{},{}", test.name, test.value, test.context).unwrap();
    }

    // Create XPT using our writer
    let xpt_path = "target/test_float_validation.xpt";
    let file = File::create(xpt_path).expect("Failed to create XPT");
    let writer = BufWriter::new(file);
    let mut xpt = XptWriter::new(writer, "FLOAT");

    xpt.write_header(&[
        ("NAME", "Char"),
        ("VALUE", "Num"),
        ("CONTEXT", "Char"),
    ]).expect("Header write failed");

    for test in TEST_CASES {
        xpt.write_row(&[
            test.name.to_string(),
            test.value.to_string(),
            test.context.to_string(),
        ]).expect("Row write failed");
    }

    println!("\n✅ Generated validation files:");
    println!("   CSV: {}", csv_path);
    println!("   XPT: {}", xpt_path);
    println!("\nValidate with Python:");
    println!("   import pandas as pd");
    println!("   csv_data = pd.read_csv('{}')", csv_path);
    println!("   xpt_data = pd.read_sas('{}', format='xport')", xpt_path);
    println!("   print((csv_data['VALUE'] - xpt_data['VALUE']).abs().max())");
}

#[test]
fn test_ibm_370_known_values() {
    // Test against documented IBM 370 conversions
    // Source: IBM System/370 Principles of Operation manual

    let test_cases = vec![
        // (input, expected_bytes)
        (0.0, vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        (1.0, vec![0x41, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        (-1.0, vec![0xC1, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        (0.5, vec![0x40, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
    ];

    for (val, bytes) in test_cases {
        let decoded = ibm370_to_f64(&bytes);
        assert!((decoded - val).abs() < 1e-10, "Failed to decode known value: {}", val);
    }
}
