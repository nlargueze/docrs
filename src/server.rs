//! File server

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

use axum::{
    body,
    http::{self, uri::PathAndQuery},
    response::{self, sse},
    routing::{any, get},
    Extension, Router,
};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tower::ServiceExt;
use tower_http::services::ServeDir;

/// Extension type for root folder
#[derive(Debug, Clone)]
struct RootFolder(PathBuf);

/// Extension type for SSE event receiver
#[derive(Debug, Clone)]
struct SseSender(broadcast::Sender<sse::Event>);

/// Starts a file server
pub async fn start_server(
    folder: &PathBuf,
    addr: &SocketAddr,
    tx_event: broadcast::Sender<sse::Event>,
) -> () {
    let root_folder = RootFolder(folder.clone());
    let sse_sender = SseSender(tx_event);

    let app = Router::new()
        .route("/__sse__", any(sse_handler))
        .fallback(get(fs_handler))
        .layer(Extension(root_folder))
        .layer(Extension(sse_sender));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/// Handles static files requests
async fn fs_handler(
    Extension(root_folder): Extension<RootFolder>,
    uri: http::Uri,
) -> response::Response {
    // try URI as is
    match request_file(&root_folder.0, &uri).await {
        FileRequestStatus::Found(res) => {
            return res;
        }
        FileRequestStatus::Error(res) => {
            return res;
        }
        FileRequestStatus::NotFound(_) => {}
    }

    // try by adding .html
    let new_uri = append_to_uri(&uri, ".html");
    match request_file(&root_folder.0, &new_uri).await {
        FileRequestStatus::Found(res) => {
            return res;
        }
        FileRequestStatus::Error(res) => {
            return res;
        }
        FileRequestStatus::NotFound(_) => {}
    }

    // try by adding /index.html
    let new_uri = append_to_uri(&uri, "/index.html");
    match request_file(&root_folder.0, &new_uri).await {
        FileRequestStatus::Found(res) => {
            return res;
        }
        FileRequestStatus::Error(res) => {
            return res;
        }
        FileRequestStatus::NotFound(_) => {}
    }

    // not found response
    response::Response::builder()
        .status(http::StatusCode::NOT_FOUND)
        .body(body::boxed("Not found error".to_string()))
        .unwrap()
}

/// File request status
pub enum FileRequestStatus {
    /// File is not found
    NotFound(response::Response<body::BoxBody>),
    /// File is found
    Found(response::Response<body::BoxBody>),
    /// Error
    Error(response::Response<body::BoxBody>),
}

/// Requests a file
async fn request_file<P: AsRef<Path>>(root_folder: P, uri: &http::Uri) -> FileRequestStatus {
    let server_dir = ServeDir::new(root_folder);
    // try URI as is
    let req = http::Request::builder()
        .uri(uri)
        .body(body::Body::empty())
        .unwrap();
    match server_dir.oneshot(req).await {
        Ok(res) => {
            if res.status() == http::StatusCode::NOT_FOUND {
                FileRequestStatus::NotFound(res.map(body::boxed))
            } else {
                FileRequestStatus::Found(res.map(body::boxed))
            }
        }
        Err(_) => {
            let res = response::Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(body::boxed("Internal server error".to_string()))
                .unwrap();
            FileRequestStatus::Error(res)
        }
    }
}

/// Modifies the path of a URI
fn append_to_uri(uri: &http::Uri, appended: &str) -> http::Uri {
    let mut parts = uri.clone().into_parts();
    if let Some(pq) = uri.path_and_query().cloned() {
        let mut path_query_str = pq.path().to_string();
        path_query_str.push_str(appended);
        if let Some(q) = pq.query() {
            path_query_str.push('?');
            path_query_str.push_str(q);
        }
        parts.path_and_query = Some(path_query_str.parse::<PathAndQuery>().unwrap());
    }

    http::Uri::from_parts(parts).unwrap()
}

/// Handles the SSE events
async fn sse_handler(Extension(sse_tx): Extension<SseSender>) -> impl response::IntoResponse {
    let sse_rx = sse_tx.0.subscribe();
    let stream = BroadcastStream::new(sse_rx);

    // // test if the SSE event is received
    // let mut sse_rx_test = sse_tx.0.subscribe();
    // tokio::task::spawn(async move {
    //     loop {
    //         let event = sse_rx_test.recv().await.unwrap();
    //         eprintln!("[sse_handler] SSE event received: {:#?}", event);
    //     }
    // });

    sse::Sse::new(stream).keep_alive(sse::KeepAlive::default())
}
