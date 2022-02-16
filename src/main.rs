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
        }
        Primitive::String(v) => {
            // let v = v.into_string().unwrap();
            let value_result = v.as_str();
            match value_result {
                Ok(v_str) => {
                    println!("String: '{}' / {:?}", v_str, v);
                }
                Err(e) => {
                    println!("Err: {:?} - {:?}", e, v)
                }
            }
        }
        _ => {
            println!("Type: {:?}", primitive);
            ()
        }
    }
}

fn main() -> Result<(), PdfError> {
    let now = SystemTime::now();

    let file = File::<Vec<u8>>::open("./VULN-20220209.12.pdf").unwrap();

    let p = file.num_pages();
    println!("Pages: {}", p);
    let first_page = file.get_page(0).unwrap();

    if let Some(content) = &first_page.contents {
        for oper in content.operations.iter() {
            match oper.operator.as_str() {
                "TJ" => {
                    for v in &oper.operands {
                        print_primitives(v);
                    }
                    // let operands: Vec<String> = (&oper.operands).into_iter().map(|p| p.try_into()).collect();
                    // println!("{:?} - {:?}", oper.operator.as_str(), operands)
                }
                _ => {
                    // println!("Operator: {}", v);
                    ()
                }
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
