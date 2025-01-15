use crate::error::IoError;
use log::error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{ReadHalf, WriteHalf};

#[inline]
pub async fn write_all(writer: &mut WriteHalf<'_>, msg: &str) -> Result<(), IoError> {
    let result = writer.write_all(msg.as_bytes()).await;
    match result {
        Err(_) => {
            error!("Error writing message: {}", msg);
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
            error!("Error reading from channel: {:?}", line);
            Err(IoError::UserDisconnected)
        }
        _ => Ok(()),
    }
}
