use log::error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{ReadHalf, WriteHalf};

#[inline]
pub async fn write_all(writer: &mut WriteHalf<'_>, msg: &str) {
    match writer.write_all(msg.as_bytes()).await {
        Err(_) => {
            error!("Failed to send '{}'", msg);
        }
        _ => {}
    }
}

#[inline]
pub async fn read_line<'a>(reader: &'a mut BufReader<ReadHalf<'_>>, mut line: &mut String) {
    match reader.read_line(&mut line).await {
        Err(_) => {
            error!("Error reading from channel: {:?}", line);
        }
        _ => {}
    };
}
