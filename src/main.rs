extern crate pdf;

use std::env::args;
use std::time::SystemTime;
use std::fs;
use std::collections::HashMap;

use pdf::file::File;
use pdf::object::*;
use pdf::primitive::Primitive;
use pdf::error::PdfError;

fn main() -> Result<(), PdfError> {
    let now = SystemTime::now();

    let file = File::<Vec<u8>>::open("./VULN-20220209.12.pdf").unwrap();
    if let Some(ref info) = file.trailer.info_dict {
        let title: Option<String> = info.get("Title").and_then(|p| Some(p.try_into().unwrap()));
        let author: Option<String> = info.get("Author").and_then(|p| Some(p.try_into().unwrap()));

        let descr = match (title, author) {
            (Some(title), None) => title,
            (None, Some(author)) => format!("[no title] – {}", author),
            (Some(title), Some(author)) => format!("{} – {}", title, author),
            _ => "PDF".into(),
        };
        println!("{}", descr);
    }

    for page in file.pages() {
        let page = page.unwrap();
        let contents = page.contents;

        let resources = page.resources().unwrap();
        for (i, &font) in resources.fonts.values().enumerate() {
            let font = file.get(font)?;
            let name = &font.name;
        }
    }

    if let Ok(elapsed) = now.elapsed() {
        println!(
            "Time: {}s",
            elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9
        );
    }
    Ok(())
}
