use axum::{
    extract::Path,
    http::{header, HeaderMap},
    response::IntoResponse,
    routing::get,
    Router, Server,
};
use mimalloc::MiMalloc;
use std::{collections::HashMap, net::SocketAddr};
use tokio::{
    fs::{read_dir, File},
    io::AsyncReadExt,
    sync::OnceCell,
};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
static FILES: OnceCell<HashMap<String, &[u8]>> = OnceCell::const_new();

#[tokio::main]
async fn main() {
    let mut paths = read_dir("./assets").await.unwrap();
    let mut files = HashMap::new();
    loop {
        match paths.next_entry().await.unwrap() {
            Some(path) => {
                let mut file = File::open(path.path()).await.unwrap();
                let file_name = path.file_name().into_string().unwrap();
                let mut file_buffer = Vec::new();
                file.read_to_end(&mut file_buffer).await.unwrap();
                let leaked_buffer = Vec::leak(file_buffer);
                files.insert(file_name, &*leaked_buffer);
            }
            None => {
                files.shrink_to_fit();
                FILES.set(files).unwrap();
                break;
            }
        }
    }
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
    let file_cache = *files.get(&req_filename).unwrap();
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
    (headers, file_cache)
}
