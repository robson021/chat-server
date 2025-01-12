use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;

const HOST: &str = "localhost:8080";

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let listener = TcpListener::bind(HOST).await.unwrap();
    let (tx, _rx) = broadcast::channel::<(String, SocketAddr)>(8);

    println!("Running on {}", HOST);

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => handle_connection(socket, addr, tx.clone()),
            Err(e) => eprintln!("Could not get client: {:?}", e),
        }
    }
}

fn handle_connection(mut socket: TcpStream, addr: SocketAddr, tx: Sender<(String, SocketAddr)>) {
    tokio::spawn(async move {
        println!("New connection from {:?}", addr);

        let (reader, mut writer) = socket.split();
        let mut rx = tx.subscribe();

        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            select! {
                result = reader.read_line(&mut line) => {
                    if result.is_err() || result.unwrap() == 0 {
                        println!("Client disconnected");
                        break;
                    }
                    let msg = (line.clone(), addr);
                    tx.send(msg).unwrap();
                    line.clear();
                }
                result = rx.recv() => {
                    let (msg, sender_addr) = result.unwrap();
                    if sender_addr != addr {
                       writer.write_all(msg.as_bytes()).await.unwrap();
                    }
                }
            }
        }
    });
}
