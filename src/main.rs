extern crate pdf;

use std::time::SystemTime;

use pdf::error::PdfError;
use pdf::file::File;

fn main() -> Result<(), PdfError> {
    let now = SystemTime::now();

    let file = File::<Vec<u8>>::open("./VULN-20220209.12.pdf").unwrap();
    if let Some(ref info) = file.trailer.info_dict {
        let title: Option<String> = info.get("Title").map(|p| p.try_into().unwrap());
        let author: Option<String> = info.get("Author").map(|p| p.try_into().unwrap());

        let descr = match (title, author) {
            (Some(title), None) => title,
            (None, Some(author)) => format!("[no title] – {}", author),
            (Some(title), Some(author)) => format!("{} – {}", title, author),
            _ => "PDF".into(),
        };
        println!("{}", descr);
    }

    if let Ok(elapsed) = now.elapsed() {
        println!(
            "Time: {}s",
            elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9
        );
    }
    Ok(())
}
