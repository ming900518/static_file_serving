use axum::{
    extract::Path,
    http::{header, HeaderMap},
    response::IntoResponse,
    routing::get,
    Router, Server,
};
use mimalloc::MiMalloc;
use std::{
    fs::{read_dir, File},
    io::Read,
    net::SocketAddr,
};
use tokio::sync::OnceCell;

#[derive(Debug)]
struct FileCache {
    file_name: String,
    file: &'static [u8],
}

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
static FILES: OnceCell<Vec<FileCache>> = OnceCell::const_new();

#[tokio::main]
async fn main() {
    let paths = read_dir("./assets").unwrap();
    let mut files = Vec::new();
    for path_result in paths {
        let path = path_result.unwrap();
        let mut file = File::open(path.path()).unwrap();
        let file_name = path.file_name().into_string().unwrap();
        let mut file_buffer = Vec::new();
        file.read_to_end(&mut file_buffer).unwrap();
        let leaked_buffer = Vec::leak(file_buffer);
        let file_cache = FileCache {
            file_name,
            file: leaked_buffer,
        };
        files.push(file_cache);
    }
    FILES.set(files).unwrap();
    let router = Router::new()
        .route("/sendfile/:req_filename", get(sendfile_api))
        .into_make_service();

    let addr = SocketAddr::from(([0, 0, 0, 0], 1370));
    println!("SSL disabled. Listening on {}", addr);
    Server::bind(&addr)
        .serve(router)
        .await
        .expect("Server startup failed.");
}

async fn sendfile_api(Path(req_filename): Path<String>) -> impl IntoResponse {
    let files = FILES.get().unwrap();
    let file_cache = files
        .iter()
        .find(|file_cache| file_cache.file_name == req_filename)
        .unwrap();
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
    (headers, file_cache.file)
}

