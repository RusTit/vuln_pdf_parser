use glob::glob;
use lopdf::Document;
use pdf_extract::{output_doc, OutputDev, OutputError, PlainTextOutput};
use regex::Regex;
use std::fmt::Debug;
use std::fs::File;
use std::fs::{create_dir_all, read_to_string, rename, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const PDF_PATTERN: &str = "*.pdf";
pub fn get_pdf_files_in_directory(directory: Option<String>) -> Vec<PathBuf> {
    let directory_to_search = match directory {
        Some(d) => d,
        None => String::from("."),
    };
    let mut pattern = PathBuf::new();
    pattern.push(directory_to_search);
    pattern.push(PDF_PATTERN);
    let pattern = pattern.display().to_string();
    log::debug!("Pattern to search pdf: {}", pattern);
    let files = glob(&pattern).expect("Failed to read glob pattern");
    let pdf_files: Vec<PathBuf> = files.map(|f| f.unwrap()).collect();
    log::debug!("Found {} pdf files in current directory.", pdf_files.len());
    return pdf_files;
}

#[derive(Debug, Default)]
pub struct Vuln {
    file_name: String,
    description: Option<String>,
    category: Option<String>,
    products: Option<String>,
}

const DESCRIPTION_BLOCK: &str = "Наличие обновления";
const CATEGORY_BLOCK: &str = "Категория уязвимого продукта";
const PRODUCT_BLOCK: &str = "Уязвимый продукт";
pub fn parse_txt(output_file: &Path) -> Option<Vuln> {
    let file_content = read_to_string(&output_file).unwrap();
    log::debug!(
        "File {} has {} content length",
        output_file.to_path_buf().display(),
        file_content.len()
    );
    let mut lines = file_content.lines();
    let mut result = Vuln {
        file_name: String::from(output_file.file_name().unwrap().to_str().unwrap()),
        ..Default::default()
    };
    while let Some(line) = lines.next() {
        let line = line.trim();

        if line.starts_with(DESCRIPTION_BLOCK) {
            while let Some(line) = lines.next() {
                let line = line.trim();
                if !line.is_empty() {
                    result.description = Some(String::from(line));
                    break;
                }
            }
        } else if line.starts_with(CATEGORY_BLOCK) {
            let mut buff = String::new();
            buff.push_str(line.strip_prefix(CATEGORY_BLOCK).unwrap().trim());
            while let Some(line) = lines.next() {
                let line = line.trim();
                if line.is_empty() {
                    break;
                }
                buff.push(' ');
                buff.push_str(line);
            }
            result.category = Some(buff);
        } else if line.starts_with(PRODUCT_BLOCK) {
            let mut buff = String::new();
            buff.push_str(line.strip_prefix(PRODUCT_BLOCK).unwrap().trim());
            while let Some(line) = lines.next() {
                let line = line.trim();
                if line.is_empty() {
                    break;
                }
                buff.push(' ');
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

pub fn convert_pdf_into_txt(output_file_path: &Path, path: &Path) -> Result<(), OutputError> {
    let mut output_file =
        BufWriter::new(File::create(output_file_path).expect("could not create output"));
    let mut output: Box<dyn OutputDev> = Box::new(PlainTextOutput::new(
        &mut output_file as &mut dyn std::io::Write,
    ));
    let doc = Document::load(&path).unwrap();
    return output_doc(&doc, output.as_mut());
}

pub fn save_report(v: &Vuln, folder_path: &str) {
    let mut report_file_path = PathBuf::new();
    log::debug!("Saving report result into folder: {}", folder_path);
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
    writeln!(
        file,
        "Description: {}",
        v.description.as_ref().unwrap_or(&stub)
    )
    .unwrap();
    writeln!(file, "Category: {}", v.category.as_ref().unwrap_or(&stub)).unwrap();
    writeln!(file, "Products: {}", v.products.as_ref().unwrap_or(&stub)).unwrap();
    writeln!(file).unwrap();
}

pub fn process_pdf_files(files: &[PathBuf]) {
    let re = Regex::new(r"-(\d+)").unwrap();
    for pdf_file in files {
        log::debug!("Processing {} file", pdf_file.display());
        let path = Path::new(pdf_file);
        let filename = path.file_name().expect("expected a filename");
        let captures = re.captures(filename.to_str().unwrap()).unwrap();
        if captures.len() == 1 {
            log::warn!("PDF file name ({}) has invalid format", pdf_file.display());
            continue;
        }
        let folder_name = &captures[1];
        let folder_path = format!("./{}", folder_name);
        let result = create_dir_all(&folder_path);
        if let Err(e) = result {
            log::warn!("Unable to create output folder: {} {}", folder_name, e);
            continue;
        }
        let mut output_file_path = PathBuf::new();
        output_file_path.push(&folder_path);
        output_file_path.push(&filename);
        output_file_path.set_extension(&EXTENSION);

        let result = convert_pdf_into_txt(&output_file_path, path);
        if let Err(e) = result {
            log::warn!("Unable to convert pdf into txt: {}", e);
            continue;
        }
        if let Some(v) = parse_txt(&output_file_path) {
            save_report(&v, &folder_path);
            let mut pdf_file_result_path = PathBuf::new();
            pdf_file_result_path.push(&folder_path);
            pdf_file_result_path.push(&filename);
            let result = rename(&pdf_file, &pdf_file_result_path);
            if let Err(e) = result {
                log::warn!("Unable to move file: {}", e);
                continue;
            }
        }
    }
}
