use std::fs;
use std::path::PathBuf;

use sotto_lib::demo_pipeline;

fn main() {
    let data_dir = std::env::var("SOTTO_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/sotto-demo-data")
        });
    if data_dir.exists() {
        let _ = fs::remove_dir_all(&data_dir);
    }
    if let Err(err) = fs::create_dir_all(&data_dir) {
        eprintln!("sotto-demo failed: {err}");
        std::process::exit(1);
    }
    match demo_pipeline(&data_dir) {
        Ok(report) => println!("{}", serde_json::to_string_pretty(&report).expect("json")),
        Err(err) => {
            eprintln!("sotto-demo failed: {err}");
            std::process::exit(1);
        }
    }
}
