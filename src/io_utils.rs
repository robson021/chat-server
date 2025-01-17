use crate::cache::{ChatHistory, SharedClientCache, Socket};
use crate::error::IoError;
use chrono::Local;
use log::{debug, info};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;

#[inline]
pub async fn write_all(writer: &mut WriteHalf<'_>, msg: &str) -> Result<(), IoError> {
    let result = writer.write_all(msg.as_bytes()).await;
    match result {
        Err(_) => {
            debug!("Error writing message: {}", msg);
            Err(IoError::CouldNotWrite)
        }
        _ => Ok(()),
    }
}

#[inline]
pub async fn read_line(
    reader: &mut BufReader<ReadHalf<'_>>,
    line: &mut String,
) -> Result<(), IoError> {
    match reader.read_line(line).await {
        Err(_) => {
            debug!("Error reading from channel: {:?}", line);
            Err(IoError::UserDisconnected)
        }
        _ => Ok(()),
    }
}

#[inline]
pub async fn send_msg_update_chat_history(
    msg: &str,
    addr: SocketAddr,
    tx: &Sender<(String, SocketAddr)>,
    user_cache: &Arc<Mutex<SharedClientCache>>,
    chat_history: &Arc<Mutex<ChatHistory>>,
) {
    let formatted_text = get_response_message(msg, user_cache, addr.to_string()).await;
    let msg = (formatted_text.clone(), addr);
    chat_history.lock().await.insert(formatted_text.clone());
    tx.send(msg).unwrap();
}

#[inline]
async fn get_response_message(
    msg: &str,
    client_cache: &Arc<Mutex<SharedClientCache>>,
    socket: Socket,
) -> String {
    let time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mut string = format!("|{}| ", time);

    let id = match client_cache.lock().await.get(&socket) {
        Some(id) => format!("[{}]", id),
        None => "[unknown]".to_owned(),
    };

    string.push_str(&id);
    string.push_str(": ");
    string.push_str(msg.trim());
    info!("Message: {}", string);
    string.push_str("\r\n");
    string
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache;
    use crate::cache::UserID;

    #[tokio::test]
    async fn should_build_response_msg() {
        // given
        let socket: Socket = "socket-id".into();
        let id: UserID = "test-id".into();

        let cache = cache::SharedClientCache::new_cache();
        cache.lock().await.insert(socket.clone(), id);

        // when
        let msg = get_response_message("base-msg", &cache, socket).await;

        // then
        assert!(msg.ends_with("| [test-id]: base-msg\r\n"));
    }
}
