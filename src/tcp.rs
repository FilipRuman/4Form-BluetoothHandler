use std::io::BufReader;
use std::io::Write;

use std::io::BufRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
const TCP_ADDRESS: &str = "127.0.0.1:2137";

pub async fn create_stream() -> TcpStream {
    println!("awaiting for tcp connection....");
    let listener = TcpListener::bind(TCP_ADDRESS).await.unwrap();
    let stream = listener.accept().await.unwrap();
    // has to be later changed
    stream.0
}
pub fn read_tcp_data(stream: &mut TcpStream) -> Option<String> {
    println!("read_tcp_data");
    let mut output = vec![0; 1024];

    match stream.try_read(&mut output) {
        Ok(_) => Some(str::from_utf8(&output).unwrap().to_string()),
        Err(_) => None,
    }
}
pub async fn send_tcp_data(stream: &mut TcpStream, data: String) {
    stream.write_all(data.as_bytes()).await.unwrap();
}
