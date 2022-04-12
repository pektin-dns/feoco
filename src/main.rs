use feoco::{recursive_read_dir, BASE_PATH};
use flate2::Compression;
use hashbrown::HashMap;
use hyper::{
    header::{HeaderName, HeaderValue},
    HeaderMap,
};
use std::convert::Infallible;
use std::str::FromStr;

use flate2::write::GzEncoder;

use hyper::service::{make_service_fn, service_fn};
use lazy_static::lazy_static;
use std::io::Write;

use hyper::{Body, Request, Response, Server};

mod config;
use crate::config::{read_config, Config};
mod lib;
use crate::lib::COMPRESSABLE_MIME_TYPES;

lazy_static! {
    static ref CONFIG: Config = read_config();
    static ref PAGES: (HashMap<String, Vec<u8>>, HashMap<String, String>) = read_to_memory();
    static ref DOCUMENT_MAP: HeaderMap = create_header_map(HeaderMapType::Document);
    static ref ALL_MAP: HeaderMap = create_header_map(HeaderMapType::All);
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // reference variables declared with lazy_static because they are initialized on first access
    let _ = &PAGES.0.len();
    let _ = &CONFIG.headers;
    let _ = &DOCUMENT_MAP.len();
    let _ = &ALL_MAP.len();

    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });

    let addr = ([0, 0, 0, 0], 80).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);
    //.with_graceful_shutdown(shutdown_signal())
    server.await?;

    Ok(())
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let fsmap = &PAGES.0;
    let on_disk_map = &PAGES.1;

    let mut path = req.uri().path();
    let on_disk = on_disk_map.contains_key(path);
    let in_memory = fsmap.contains_key(path);
    let request_headers = req.headers();
    let accept_gzip = request_headers
        .get("accept-encoding")
        .unwrap_or(&HeaderValue::from_static(""))
        .to_str()
        .unwrap()
        .contains("gzip");

    let mut res = Response::builder().status(200);

    if !in_memory && !on_disk {
        path = "/index.html";
    };

    let content_type = mime_guess::from_path(path).first_or_octet_stream();

    if content_type == "text/html" {
        res.headers_mut().unwrap().extend(DOCUMENT_MAP.clone());
    } else {
        res.headers_mut().unwrap().extend(ALL_MAP.clone());
    }

    res = res.header("content-type", content_type.as_ref());

    if on_disk {
        let file = std::fs::read(on_disk_map.get(path).unwrap()).unwrap();

        let res = res.body(Body::from(file)).unwrap();
        Ok(res)
    } else {
        let access_path = if accept_gzip && COMPRESSABLE_MIME_TYPES.contains(&content_type.as_ref())
        {
            res = res.header("content-encoding", "gzip");
            format!("{}_gz", path)
        } else {
            String::from(path)
        };

        let res = res
            .body(Body::from(fsmap.get(&access_path).unwrap().clone()))
            .unwrap();
        Ok(res)
    }
}
pub enum HeaderMapType {
    Document,
    All,
}

pub fn create_header_map(map_type: HeaderMapType) -> HeaderMap<HeaderValue> {
    let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
    let config = &CONFIG;
    if matches!(map_type, HeaderMapType::Document) {
        for header in &config.headers.document {
            headers.insert(
                HeaderName::from_str(header.0).unwrap(),
                HeaderValue::from_str(header.1).unwrap(),
            );
        }
    }
    for header in &config.headers.all {
        headers.insert(
            HeaderName::from_str(header.0).unwrap(),
            HeaderValue::from_str(header.1).unwrap(),
        );
    }

    headers
}

pub fn read_to_memory() -> (HashMap<String, Vec<u8>>, HashMap<String, String>) {
    let mut fsmap: HashMap<String, Vec<u8>> = HashMap::new();
    let mut not_in_mem_map: HashMap<String, String> = HashMap::new();
    let config = &CONFIG;
    let mut file_content_size: u128 = 0;
    let mut file_content_size_compressed: u128 = 0;

    for entry in recursive_read_dir(BASE_PATH) {
        if entry.file_type().unwrap().is_file() {
            let path = entry.path();
            let path_str = path.to_str().unwrap();
            let file_content = std::fs::read(path_str).unwrap();
            let no_memory = config.no_memory.iter().any(|nm| path_str.contains(nm));

            let path_repl_base_path = String::from(path_str).replace(BASE_PATH, "");

            let is_compressable_type = COMPRESSABLE_MIME_TYPES.contains(
                &mime_guess::from_path(path_str)
                    .first_or_octet_stream()
                    .as_ref(),
            );

            if no_memory {
                not_in_mem_map.insert(path_repl_base_path, path_str.into());
            } else {
                file_content_size += file_content.len() as u128;
                if is_compressable_type {
                    let mut z = GzEncoder::new(Vec::new(), Compression::best());
                    z.write_all(file_content.as_slice()).unwrap();

                    let file_content_gz = z.finish().unwrap();
                    file_content_size_compressed += file_content_gz.len() as u128;

                    fsmap.insert(format!("{}_gz", path_repl_base_path), file_content_gz);
                }
                fsmap.insert(path_repl_base_path, file_content);
            }
        }
    }

    println!(
        "In memory size: {} MiB\nIn memory size compressed: {} MiB\nTotal memory size: {} MiB",
        file_content_size / 1024 / 1024,
        file_content_size_compressed / 1024 / 1024,
        (file_content_size + file_content_size_compressed) / 1024 / 1024
    );

    (fsmap, not_in_mem_map)
}
