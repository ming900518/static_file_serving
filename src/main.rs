use std::{
    fs::{read_dir, File},
    io::Read,
    net::SocketAddr,
};

use axum::{
    body::StreamBody,
    extract::{Path, State},
    http::{header, HeaderMap},
    routing::get,
    Router,
};
use tokio_util::io::ReaderStream;

#[tokio::main]
async fn main() {
    let paths = read_dir("./assets").unwrap();
    let mut buffer = Vec::new();
    paths.into_iter().for_each(|path_result| {
        let path = path_result.unwrap();
        let mut file = File::open(path.path()).unwrap();
        let filename = path.file_name().into_string().unwrap();
        let mut file_bytes = Vec::new();
        file.read_to_end(&mut file_bytes).unwrap();
        let leaked_file: &'static [u8] = &*Vec::leak(file_bytes);
        buffer.append(&mut vec![(filename, leaked_file)]);
    });
    let leaked_buffer: &'static[(String, &'static[u8])] = Vec::leak(buffer);

    let router = Router::new()
        .route("/sendfile/:req_filename", get(sendfile_api))
        .with_state(leaked_buffer)
        .into_make_service();

    let addr = SocketAddr::from(([0, 0, 0, 0], 1370));
    println!("SSL disabled. Listening on {}", addr);
    axum_server::bind(addr)
        .serve(router)
        .await
        .expect("Server startup failed.");
}

async fn sendfile_api(
    State(buffer): State<&'static[(String, &'static [u8])]>,
    Path(req_filename): Path<String>,
) -> (HeaderMap, StreamBody<ReaderStream<&[u8]>>) {
    let file = buffer.iter().find(|(filename, _)| filename == &req_filename).unwrap().to_owned();
    let reader_stream = ReaderStream::new(file.1);
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
