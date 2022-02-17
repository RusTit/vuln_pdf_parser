extern crate lopdf;
extern crate pdf_extract;

use std::time::SystemTime;
use vuln_pdf_parser::{get_pdf_files_in_directory, process_pdf_files};

fn main() {
    let now = SystemTime::now();
    let files = get_pdf_files_in_directory();
    process_pdf_files(&files);

    if let Ok(elapsed) = now.elapsed() {
        println!(
            "Time: {}s",
            elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9
        );
    }
}
