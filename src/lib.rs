use std::fs::DirEntry;

use flate2::write::GzEncoder;
use flate2::Compression;
use hashbrown::HashMap;
use std::io::Write;

pub const BASE_PATH: &str = "/public";

pub const COMPRESSABLE_MIME_TYPES: [&str; 15] = [
    "text/css",
    "application/javascript",
    "text/html",
    "image/svg+xml",
    "text/xml",
    "text/plain",
    "application/json",
    "application/yaml",
    "application/yml",
    "application/toml",
    "text/markdown",
    "application/wasm",
    "application/json-p",
    "text/javascript",
    "text/css",
];

pub fn recursive_read_dir(path: &str) -> Vec<DirEntry> {
    recursive_read_dir_inner(path).unwrap()
}

pub fn recursive_read_dir_inner(path: &str) -> Result<Vec<DirEntry>, Box<dyn std::error::Error>> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            entries.extend(recursive_read_dir_inner(path.to_str().unwrap()).unwrap());
        } else {
            entries.push(entry);
        }
    }
    Ok(entries)
}

pub fn read_to_memory() -> HashMap<String, Vec<u8>> {
    let mut fsmap: HashMap<String, Vec<u8>> = HashMap::new();

    let mut file_content_size: u128 = 0;
    let mut file_content_size_compressed: u128 = 0;

    for entry in recursive_read_dir(BASE_PATH) {
        if entry.file_type().unwrap().is_file() {
            let path = entry.path();
            let path_str = path.to_str().unwrap();
            let file_content = std::fs::read(path_str).unwrap();

            file_content_size += file_content.len() as u128;
            if COMPRESSABLE_MIME_TYPES.contains(
                &mime_guess::from_path(path_str)
                    .first_or_octet_stream()
                    .as_ref(),
            ) {
                println!("{:?}", path_str);

                let mut z = GzEncoder::new(Vec::new(), Compression::best());
                z.write_all(file_content.as_slice()).unwrap();

                let file_content_gz = z.finish().unwrap();
                file_content_size_compressed += file_content_gz.len() as u128;

                fsmap.insert(
                    format!("{}_gz", String::from(path_str).replace(BASE_PATH, "")),
                    file_content_gz,
                );
            }
            fsmap.insert(String::from(path_str).replace(BASE_PATH, ""), file_content);
        }
    }

    println!(
        "In memory size: {} MiB\nIn memory size compressed: {} MiB\nTotal memory size: {} MiB",
        file_content_size / 1024 / 1024,
        file_content_size_compressed / 1024 / 1024,
        (file_content_size + file_content_size_compressed) / 1024 / 1024
    );

    fsmap
}
