pub mod peripherals_tcp_parser;
pub mod tcp_parser;

use std::error::Error;

use anyhow::Context;
use anyhow::Result;
use spdlog::prelude::*;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
const TCP_ADDRESS: &str = "127.0.0.1:2137";

pub async fn create_stream() -> Result<TcpStream> {
    info!("awaiting for tcp connection....");
    let listener = TcpListener::bind(TCP_ADDRESS)
        .await
        .context("create_stream: create listener")?;
    let stream = listener.accept().await?;
    Ok(stream.0)
}
pub(super) fn read_tcp_data(stream: &mut TcpStream) -> Option<String> {
    let mut output = vec![0; 1024];

    match stream.try_read(&mut output) {
        Ok(_) => Some(str::from_utf8(&output).unwrap().to_string()),
        Err(_) => None,
    }
}
pub(super) async fn send_tcp_data(stream: &mut TcpStream, mut data: String) {
    data += "\n";
    match stream.write_all(data.as_bytes()).await {
        Ok(out) => out,
        Err(err) => {
            warn!("send_tcp_data did not succeed because:{err:?}")
        }
    };
}
