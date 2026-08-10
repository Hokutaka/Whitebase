//! Whitebase HTTP APIサーバーの起動Hostです。

#![forbid(unsafe_code)]

use std::error::Error;

const SERVER_ADDRESS: &str = "127.0.0.1:1430";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = tokio::net::TcpListener::bind(SERVER_ADDRESS).await?;

    println!("[Whitebase Server] Listening on http://{SERVER_ADDRESS}");

    axum::serve(listener, whitebase_http_api::router()).await?;

    Ok(())
}
