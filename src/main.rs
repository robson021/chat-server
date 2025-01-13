mod user_cache;

use crate::user_cache::{Shared, Socket};
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
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

    // todo: concurrent map based on read-write lock will be more performant
    let user_cache: Arc<Mutex<Shared>> = user_cache::new_cache();

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                handle_connection(socket, addr, tx.clone(), Arc::clone(&user_cache))
            }
            Err(e) => eprintln!("Could not get client: {:?}", e),
        }
    }
}

fn handle_connection(
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

    match user_cache.lock() {
        Ok(mut mutex) => {
            println!("New user added to the cache: {}.", id);
            mutex.clients.insert(addr.to_string(), id.clone());
        }
        Err(_) => {
            eprintln!("Could not connect add new user.");
            return;
        }
    }
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
                        let user = user_cache.lock().unwrap().clients.remove(&addr.to_string());
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
                       let msg = get_message(&msg, &user_cache, sender_addr.to_string());
                       writer.write_all(msg.as_bytes()).await.unwrap();
                    }
                }
            }
        }
    });
}

fn get_message(msg: &str, cache: &Arc<Mutex<Shared>>, socket: Socket) -> String {
    let mut string = match cache.lock() {
        Ok(mutex) => mutex.clients.get(&socket).unwrap().to_string(),
        Err(_) => String::from("unknown"),
    };
    string.push_str(": ");
    string.push_str(msg);
    string
}
