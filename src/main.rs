extern crate pdf;

use std::time::SystemTime;

use pdf::error::PdfError;
use pdf::file::File;
use pdf::primitive::Primitive;


fn print_primitives(primitive: &Primitive) {
    match primitive {
        Primitive::Array(arr) => {
            println!("Array start:");
            for v in arr {
                print_primitives(v);
            }
            println!("Array end.");
        },
        Primitive::String(v) => {
            // let v = v.into_string().unwrap();
            let value = v.as_str();
            match value {
                Ok(v) => {
                    println!("String: {}", v);
                },
                Err(e) => {
                    println!("Err: {:?} - {:?}", e, v)
                },
            }

        },
        _ => (),
    }
}

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

    let first_page = file.get_page(0).unwrap();

    if let Some(content) = &first_page.contents {
        for oper in content.operations.iter() {
            match (oper.operator.as_str(), oper.operands.as_slice()) {
                ("BT", _) => {}
                ("TJ", [Primitive::String(text)]) => {
                    // "Show text" - the operation that actually contains the
                    // text to be displayed.
                    println!("{:?}", text.as_str().ok());
                }
                ("TJ", _) => {
                    for v in &oper.operands {
                        print_primitives(v);
                    }
                    // let operands: Vec<String> = (&oper.operands).into_iter().map(|p| p.try_into()).collect();
                    // println!("{:?} - {:?}", oper.operator.as_str(), operands)
                }
                _ => continue,
            }
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
