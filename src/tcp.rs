pub mod device_tcp_parser;
pub mod peripherals_tcp_parser;
pub mod tcp_parser;

use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use spdlog::prelude::*;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;
const TCP_ADDRESS: &str = "127.0.0.1:2137";

pub async fn create_stream() -> Result<TcpStream> {
    info!("awaiting for tcp connection....");
    let listener = TcpListener::bind(TCP_ADDRESS)
        .await
        .context("create_stream: create listener")?;

    let stream = listener.accept().await?;
    Ok(stream.0)
}
pub async fn setup_tcp() -> (OwnedReadHalf, Sender<String>) {
    let stream = match create_stream().await {
        Ok(o) => o,
        Err(e) => {
            error!("creating tcp listener did not succeed because:{e:?}");
            panic!()
        }
    };
    let (tcp_reader, tcp_sender) = stream.into_split();
    let (tcp_writer_sender, tcp_writer_receiver) = mpsc::channel::<String>(21);
    tokio::spawn(tcp_send_loop(tcp_sender, tcp_writer_receiver));
    (tcp_reader, tcp_writer_sender)
}
pub async fn tcp_send_loop(mut tcp_sender: OwnedWriteHalf, mut receiver: Receiver<String>) {
    loop {
        if let Some(message) = receiver.recv().await {
            send_tcp_data(&mut tcp_sender, message).await;
        } else {
            // to be more efficient
            // because if there is nothing to send it will do hundreds of loops wasteful
            sleep(Duration::from_millis(1)).await;
        }
    }
}
pub(super) fn read_tcp_data(stream: &mut OwnedReadHalf) -> Option<String> {
    let mut output = vec![0; 1024];

    match stream.try_read(&mut output) {
        Ok(_) => Some(str::from_utf8(&output).unwrap().to_string()),
        Err(_) => None,
    }
}
pub(super) async fn send_tcp_data(stream: &mut OwnedWriteHalf, mut data: String) {
    data += "\n";

    match stream.write_all(data.as_bytes()).await {
        Ok(out) => out,
        Err(err) => {
            error!("send_tcp_data did not succeed because:{err:?}");
            panic!()
        }
    };
}
