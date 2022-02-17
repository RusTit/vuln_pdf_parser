use glob::glob;
use std::fmt::{Debug, write};
use std::io::{BufWriter, Write};
use std::fs::File;
use std::path::{PathBuf, Path};
use std::fs::{rename, create_dir_all, read_to_string, OpenOptions};
use lopdf::{Document};
use pdf_extract::{OutputDev,PlainTextOutput,output_doc, OutputError};
use regex::Regex;

pub fn get_pdf_files_in_directory() -> Vec<PathBuf> {
    let files = glob("./*.pdf").expect("Failed to read glob pattern");
    files.map(|f| f.unwrap()).collect()
}

#[derive(Debug, Default)]
pub struct Vuln {
    file_name: String,
    description: Option<String>,
    category: Option<String>,
    products: Option<String>
}

pub fn parse_txt(output_file: &PathBuf) -> Option<Vuln> {
    let file_content = read_to_string(&output_file).unwrap();
    let mut lines = file_content.lines();
    let mut result = Vuln::default();
    result.file_name = String::from(output_file.file_name().unwrap().to_str().unwrap());
    while let Some(line) = lines.next() {
        let line = line.trim();

        if line.starts_with("Наличие обновления") {
            while let Some(line) = lines.next() {
                let line = line.trim();
                if line.len() > 0 {
                    result.description = Some(String::from(line));
                    break;
                }
            }
        } else if line.starts_with("Категория уязвимого продукта") {
            let mut buff = String::new();
            buff.push_str(line.strip_prefix("Категория уязвимого продукта").unwrap().trim());
            while let Some(line) = lines.next() {
                let line = line.trim();
                if line.len() == 0 {
                    break;
                }
                buff.push_str(" ");
                buff.push_str(line);
            }
            result.category = Some(buff);
        } else if line.starts_with("Уязвимый продукт") {
            let mut buff = String::new();
            buff.push_str(line.strip_prefix("Уязвимый продукт").unwrap().trim());
            while let Some(line) = lines.next() {
                let line = line.trim();
                if line.len() == 0 {
                    break;
                }
                buff.push_str(" ");
                buff.push_str(line);
            }
            result.products = Some(buff);
        }
    }
    if result.category != None && result.description != None {
        return Some(result);
    }
    None
}

const EXTENSION: &str = "txt";

pub fn convert_pdf_into_txt(output_file_path: &PathBuf, path: &Path) -> Result<(), OutputError> {
    let mut output_file = BufWriter::new(File::create(output_file_path).expect("could not create output"));
    let mut output: Box<dyn OutputDev> = Box::new(PlainTextOutput::new(&mut output_file as &mut dyn std::io::Write));
    let doc = Document::load(&path).unwrap();
    return output_doc(&doc, output.as_mut());
}

pub fn save_report(v: &Vuln, folder_path: &String) {
    let mut report_file_path = PathBuf::new();
    report_file_path.push(folder_path);
    report_file_path.push("report.txt");
    let mut file = OpenOptions::new()
      .write(true)
      .append(true)
      .create(true)
      .open(report_file_path)
      .unwrap();
    writeln!(file, "Filename: {}", v.file_name).unwrap();
    let stub = String::from("=======");
    writeln!(file, "Description: {}", v.description.as_ref().unwrap_or(&stub)).unwrap();
    writeln!(file, "Category: {}", v.category.as_ref().unwrap_or(&stub)).unwrap();
    writeln!(file, "Products: {}", v.products.as_ref().unwrap_or(&stub)).unwrap();
    writeln!(file).unwrap();
}

pub fn process_pdf_files(files: &Vec<PathBuf>) {
    let re = Regex::new(r"-(\d+)").unwrap();
    for pdf_file in files {
        let path = Path::new(pdf_file);
        let filename = path.file_name().expect("expected a filename");
        let captures = re.captures(filename.to_str().unwrap()).unwrap();
        if captures.len() == 1 {
            // pdf has unknown format
            continue;
        }
        let folder_name = &captures[1];
        let folder_path = format!("./{}", folder_name);
        let result = create_dir_all(&folder_path);
        if let Err(_e) = result {
            // unable to create output
            continue;
        }
        let mut output_file_path = PathBuf::new();
        output_file_path.push(&folder_path);
        output_file_path.push(&filename);
        output_file_path.set_extension(&EXTENSION);

        let result = convert_pdf_into_txt(&output_file_path, &path);
        if let Err(_e) = result {
            // unable to convert pdf into txt
            continue;
        }
        if let Some(v) = parse_txt(&output_file_path) {
            save_report(&v, &folder_path);
        }
    }
}