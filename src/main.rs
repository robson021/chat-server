mod cache;
mod logger_config;

use crate::cache::{ChatHistory, SharedClientCache, Socket};
use log::{error, info, warn};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::broadcast::Sender;
use tokio::sync::{broadcast, Mutex};

// const HOST: &str = "0.0.0.0:8080";
const HOST: &str = "localhost:8080";

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    logger_config::setup_logger();

    let listener = TcpListener::bind(HOST).await.unwrap();
    let (tx, _rx) = broadcast::channel::<(String, SocketAddr)>(8);

    // todo: collections based on read-write lock will be more performant
    let user_cache = SharedClientCache::new_cache();
    let chat_history = ChatHistory::new_chat_history();

    info!("Running on {}", HOST);

    loop {
        chat_history.lock().await.drain(999);

        match listener.accept().await {
            Ok((socket, addr)) => {
                let user_cache = Arc::clone(&user_cache);
                let chat_history = Arc::clone(&chat_history);
                handle_connection(socket, addr, tx.clone(), user_cache, chat_history).await
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
) {
    tokio::spawn(async move {
        info!("New connection from {:?}", addr);

        let mut rx = tx.subscribe();
        let (reader, mut writer) = socket.split();

        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        writer
            .write_all("Enter nickname: ".as_bytes())
            .await
            .unwrap();
        reader.read_line(&mut line).await.unwrap();

        let id = line.trim().to_owned();
        line.clear();

        if id.len() < 3 {
            warn!("Too short id: {}", id);
        }

        info!("Adding new user to the cache: {}.", id);
        user_cache.lock().await.clients.insert(addr.to_string(), id);

        let old_messages: Vec<String> = chat_history.lock().await.history.clone().into();
        let mut old_messages = old_messages.join("");
        old_messages.push_str("+---------------Start chatting---------------+\r\n");

        writer.write_all(old_messages.as_bytes()).await.unwrap();

        loop {
            select! {
                result = reader.read_line(&mut line) => {
                    if result.is_err() || result.unwrap() == 0 {
                        let user = match user_cache.lock().await.clients.remove(&addr.to_string()) {
                            Some(id) => id,
                            None => "unknown".to_owned(),
                        };
                        info!("Client disconnected: {:?}", user);
                        break;
                    }
                    let formatted_text = get_response_message(line.as_str(), &user_cache, addr.to_string()).await;
                    let msg = (formatted_text.clone(), addr);
                    chat_history.lock().await.insert(formatted_text.clone());
                    tx.send(msg).unwrap();
                    line.clear();
                }
                result = rx.recv() => {
                    if result.is_err() {
                        error!("Error receiving from channel: {:?}", addr);
                        continue;
                    }
                    let (msg, sender_addr) = result.unwrap();
                    if sender_addr != addr {
                       if (writer.write_all(msg.as_bytes()).await).is_err() { error!("Error writing to channel: {:?}", sender_addr) };
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
