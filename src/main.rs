mod cache;
mod config;
mod error;
mod io_utils;

use crate::cache::{ChatHistory, SharedClientCache, Socket};
use log::{debug, error, info, warn};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::BufReader;
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::broadcast::Sender;
use tokio::sync::{broadcast, Mutex};

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let config = config::get_config();
    config.setup_logger();

    let host = &config.host;

    let listener = TcpListener::bind(host)
        .await
        .expect("Could not bind the host");
    let (tx, _rx) = broadcast::channel::<(String, SocketAddr)>(8);

    // todo: collections based on read-write lock will be more performant
    let user_cache = SharedClientCache::new_cache();

    let chat_history = match config.log_file {
        Some(file) => ChatHistory::from_local_log_file(&file),
        None => ChatHistory::empty_chat_history(),
    };

    info!("Running on: {}", host);

    loop {
        chat_history.lock().await.drain();

        match listener.accept().await {
            Ok((socket, addr)) => {
                let user_cache = Arc::clone(&user_cache);
                let chat_history = Arc::clone(&chat_history);
                handle_connection(
                    socket,
                    addr,
                    tx.clone(),
                    user_cache.clone(),
                    chat_history,
                    config.password.clone(),
                )
                .await;
            }
            Err(e) => error!("Could not get client: {:?}", e),
        };
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    addr: SocketAddr,
    tx: Sender<(String, SocketAddr)>,
    user_cache: Arc<Mutex<SharedClientCache>>,
    chat_history: Arc<Mutex<ChatHistory>>,
    password: Option<String>,
) {
    tokio::spawn(async move {
        info!("New connection from {:?}", addr);

        let (reader, mut writer) = socket.split();

        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        if let Some(password) = password {
            let valid_password =
                check_password(addr, &mut writer, &mut reader, &mut line, &password).await;
            if !valid_password {
                warn!("Password is invalid: {}", addr);
                return;
            }
        }

        io_utils::write_all(&mut writer, "Enter nickname: ")
            .await
            .unwrap();
        io_utils::read_line(&mut reader, &mut line).await.unwrap();

        let id = line.trim().to_owned();
        line.clear();

        if id.len() < 3 || id.len() > 32 {
            warn!("Invalid id: {}", id);
            return;
        };

        info!("Adding new user to the cache: {}.", id);
        user_cache.lock().await.insert(addr.to_string(), id);

        let old_messages: Vec<String> = chat_history.lock().await.history.clone().into();
        let mut old_messages = old_messages.join("");
        old_messages.push_str("+---------------Start chatting---------------+\r\n");

        io_utils::write_all(&mut writer, &old_messages)
            .await
            .unwrap();

        let mut rx = tx.subscribe();

        loop {
            select! {
                result = io_utils::read_line(&mut reader, &mut line) => {
                    if result.is_err() {
                        let socket: Socket  = addr.to_string();
                        let user = match user_cache.lock().await.remove(&socket) {
                            Some(id) => id,
                            None => "unknown".to_owned(),
                        };
                        info!("Client disconnected: {:?}", user);
                        break;
                    }
                    if !line.trim().is_empty() {
                        io_utils::send_msg_update_chat_history(&line, addr, &tx, &user_cache, &chat_history).await;
                    }
                    line.clear();
                }
                result = rx.recv() => {
                    if result.is_err() {
                        error!("Error receiving from channel: {:?}", addr);
                        continue;
                    }
                    let (msg, sender_addr) = result.unwrap();
                    if sender_addr != addr {
                        let _result = io_utils::write_all(&mut writer, &msg).await;
                    }
                }
            }
        }
    });
}

async fn check_password(
    addr: SocketAddr,
    writer: &mut WriteHalf<'_>,
    reader: &mut BufReader<ReadHalf<'_>>,
    line: &mut String,
    password: &str,
) -> bool {
    io_utils::write_all(writer, "Enter password: ")
        .await
        .unwrap();
    io_utils::read_line(reader, line).await.unwrap();

    debug!("Received password: {}", line);

    if password != line.trim() {
        warn!("Passwords do not match for: {}", addr);
        return false;
    }
    line.clear();
    true
}
