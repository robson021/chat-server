mod cache;
mod error;
mod host;
mod io_utils;
mod logger_config;

use crate::cache::{ChatHistory, SharedClientCache, Socket};
use log::{debug, error, info, warn};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::BufReader;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::broadcast::Sender;
use tokio::sync::{broadcast, Mutex};

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    logger_config::setup_logger();

    let args: Vec<String> = std::env::args().collect();
    debug!("{:?}", args);

    let password = match args.len() > 1 {
        true => args[1].trim().to_owned(),
        false => "".to_owned(),
    };

    let host = host::get_host();

    let listener = TcpListener::bind(host)
        .await
        .expect("Could not bind the host");
    let (tx, _rx) = broadcast::channel::<(String, SocketAddr)>(8);

    // todo: collections based on read-write lock will be more performant
    let user_cache = SharedClientCache::new_cache();
    let chat_history = ChatHistory::new_chat_history();

    info!("Running on: {}", host);

    loop {
        chat_history.lock().await.drain(999);

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
                    password.to_owned(),
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
    password: String,
) {
    tokio::spawn(async move {
        info!("New connection from {:?}", addr);

        let (reader, mut writer) = socket.split();

        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        if !password.is_empty() {
            io_utils::write_all(&mut writer, "Enter password: ")
                .await
                .unwrap();
            io_utils::read_line(&mut reader, &mut line).await.unwrap();

            debug!("Received password: {}", password);

            if password != line.trim() {
                warn!("Passwords do not match for: {}", addr);
                return;
            }
            line.clear();
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
        user_cache.lock().await.clients.insert(addr.to_string(), id);

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
                        let user = match user_cache.lock().await.clients.remove(&addr.to_string()) {
                            Some(id) => id,
                            None => "unknown".to_owned(),
                        };
                        info!("Client disconnected: {:?}", user);
                        break;
                    }
                    if !line.trim().is_empty() {
                        let formatted_text = get_response_message(line.as_str(), &user_cache, addr.to_string()).await;
                        let msg = (formatted_text.clone(), addr);
                        chat_history.lock().await.insert(formatted_text.clone());
                        tx.send(msg).unwrap();
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

async fn get_response_message(
    msg: &str,
    cache: &Arc<Mutex<SharedClientCache>>,
    socket: Socket,
) -> String {
    let id = match cache.lock().await.clients.get(&socket) {
        Some(id) => format!("[{}]", id),
        None => "[unknown]".to_owned(),
    };

    let mut string = id;
    string.push_str(": ");
    string.push_str(msg.trim());
    info!("Message: {}", string);
    string.push_str("\r\n");
    string
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::UserID;

    #[tokio::test]
    async fn should_build_response_msg() {
        // given
        let socket: Socket = "socket-id".into();
        let id: UserID = "test-id".into();

        let cache = cache::SharedClientCache::new_cache();
        cache.lock().await.clients.insert(socket.clone(), id);

        // when
        let msg = get_response_message("base-msg", &cache, socket).await;

        // then
        assert_eq!("[test-id]: base-msg\r\n", msg);
    }
}
