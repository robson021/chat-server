mod user_cache;

use crate::user_cache::{Shared, Socket};
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::broadcast::Sender;
use tokio::sync::{broadcast, Mutex};

const HOST: &str = "localhost:8080";

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let listener = TcpListener::bind(HOST).await.unwrap();
    let (tx, _rx) = broadcast::channel::<(String, SocketAddr)>(8);

    println!("Running on {}", HOST);

    // todo: concurrent map based on read-write lock will be more performant
    let user_cache: Arc<Mutex<Shared>> = user_cache::new_cache();

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                handle_connection(socket, addr, tx.clone(), Arc::clone(&user_cache)).await
            }
            Err(e) => eprintln!("Could not get client: {:?}", e),
        }
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    addr: SocketAddr,
    tx: Sender<(String, SocketAddr)>,
    user_cache: Arc<Mutex<Shared>>,
) {
    let id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    println!("Adding new user to the cache: {}.", id);
    user_cache.lock().await.clients.insert(addr.to_string(), id);

    tokio::spawn(async move {
        println!("New connection from {:?}", addr);

        let mut rx = tx.subscribe();
        let (reader, mut writer) = socket.split();

        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            select! {
                result = reader.read_line(&mut line) => {
                    if result.is_err() || result.unwrap() == 0 {
                        let user = user_cache.lock().await.clients.remove(&addr.to_string());
                        println!("Client disconnected: {:?}", user.unwrap());
                        break;
                    }
                    let msg = (line.clone(), addr);
                    tx.send(msg).unwrap();
                    line.clear();
                }
                result = rx.recv() => {
                    let (msg, sender_addr) = result.unwrap();
                    if sender_addr != addr {
                       let msg = get_message(&msg, &user_cache, sender_addr.to_string()).await;
                       writer.write_all(msg.as_bytes()).await.unwrap();
                    }
                }
            }
        }
    });
}

async fn get_message(msg: &str, cache: &Arc<Mutex<Shared>>, socket: Socket) -> String {
    let guard = cache.lock().await;
    let mut string = guard.clients.get(&socket).unwrap().clone();
    string.push_str(": ");
    string.push_str(msg);
    string
}
