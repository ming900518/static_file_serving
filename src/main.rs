#![feature(once_cell)]
use std::{
    fs::{read_dir, File},
    net::SocketAddr,
    sync::OnceLock, io::Read,
};

use axum::{
    body::StreamBody,
    extract::Path,
    http::{header, HeaderMap},
    response::IntoResponse,
    routing::get,
    Router, Server,
};
use tokio::runtime::Handle;
use tokio_util::io::ReaderStream;

#[derive(Debug)]
struct FileCache {
    file_name: String,
    file: &'static[u8],
}

static FILES: OnceLock<Vec<FileCache>> = OnceLock::new();

#[tokio::main]
async fn main() {
    tokio::task::block_in_place(move || {
        let paths = read_dir("./assets").unwrap();
        let mut files = Vec::new();
        for path_result in paths {
            let path = path_result.unwrap();
            let mut file = File::open(path.path()).unwrap();
            let file_name = path.file_name().into_string().unwrap();
            let mut file_buffer = Vec::new();
            file.read_to_end(&mut file_buffer);
            let leaked_buffer = Vec::leak(file_buffer);
            let file_cache = FileCache { file_name, file: leaked_buffer };
            files.push(file_cache);
        }
        FILES.set(files);
        Handle::current().block_on(async move {
            let router = Router::new()
                .route("/sendfile/:req_filename", get(sendfile_api))
                .into_make_service();

            let addr = SocketAddr::from(([0, 0, 0, 0], 1370));
            println!("SSL disabled. Listening on {}", addr);
            Server::bind(&addr)
                .serve(router)
                .await
                .expect("Server startup failed.");
        });
    });
}

async fn sendfile_api(Path(req_filename): Path<String>) -> impl IntoResponse {
    let files = FILES.get().unwrap();
    let file_cache = files
        .iter()
        .find(|file_cache| file_cache.file_name == req_filename)
        .unwrap();
    let reader_stream = ReaderStream::new(file_cache.file);
    let body = StreamBody::new(reader_stream);
    let mut headers = HeaderMap::new();
    headers.append(
        header::CONTENT_TYPE,
        "application/octet-stream".parse().unwrap(),
    );
    headers.append(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", req_filename)
            .parse()
            .unwrap(),
    );
    (headers, body)
}
