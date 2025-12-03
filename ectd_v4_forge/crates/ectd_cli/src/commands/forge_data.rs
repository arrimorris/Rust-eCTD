use clap::Args;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use ectd_core::sdtm::xpt_v5::XptWriter;

#[derive(Debug, Args)]
pub struct ForgeDataArgs {
    /// The raw CSV file containing clinical data (e.g., raw_ae.csv)
    #[arg(short, long)]
    pub input: PathBuf,

    /// The mapping JSON file (e.g., mappings/ae_map.json)
    #[arg(short, long)]
    pub map: PathBuf,

    /// Where to save the SAS XPT file (e.g., output/ae.xpt)
    #[arg(short, long)]
    pub output: PathBuf,

    /// The Domain Code (e.g., AE, DM)
    #[arg(short, long)]
    pub domain: String,
}

pub fn run(args: ForgeDataArgs) -> anyhow::Result<()> {
    println!("🔨 Forging Dataset: {:?}", args.output);

    // 1. Setup Input (CSV Reader)
    let mut rdr = csv::Reader::from_path(&args.input)?;
    let headers = rdr.headers()?.clone();

    // 2. Setup Output (XptWriter)
    let file = File::create(&args.output)?;
    let writer = BufWriter::new(file);
    let mut xpt = XptWriter::new(writer, &args.domain);

    // 3. Define Variables (This normally comes from the JSON map)
    // For this proof of concept, we just map CSV headers 1:1
    // In the real version, you'd read 'args.map' to enforce types/lengths.
    let mut var_defs = Vec::new();
    for h in &headers {
        var_defs.push((h, "Char")); // Default to Char for now
    }

    // 4. Write Header
    xpt.write_header(&var_defs)?;

    // 5. Stream Rows
    let mut row_count = 0;
    for result in rdr.records() {
        let record = result?;
        // Convert CSV record to Vec<String> for XptWriter
        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        xpt.write_row(&row)?;
        row_count += 1;
    }

    println!("✅ Forged {} rows into XPT v5 format.", row_count);
    Ok(())
}
