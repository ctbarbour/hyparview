use crate::ProtocolMessage;
use bytes::{Buf, BytesMut};
use std::io::Cursor;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,
    peer: SocketAddr,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(peer: SocketAddr, stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
            peer: peer,
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn read_frame(
        &mut self,
    ) -> Result<Option<ProtocolMessage>, Box<dyn std::error::Error + Send + Sync>> {
        loop {
            let mut buf = Cursor::new(&self.buffer[..]);

            // If we have data in the read buffer
            if buf.has_remaining() {
                // Get the first byte in the read buffer as the length of the message
                let length = buf.get_u8() as usize;

                // check if we have a full message in the read buffer
                if buf.remaining() >= length {
                    // parse the message from the read buffer
                    let start = buf.position() as usize;
                    let frame = serde_cbor::from_slice(&buf.get_ref()[start..length]).unwrap();

                    // discard the parsed data from the read buffer
                    self.buffer.advance(length);

                    return Ok(Some(frame));
                }
            }

            // Not enough data in the read buffer, so continue reading
            // 0 means end of stream. If we still have partial data in the buffer, then the client
            // disconnected while sending data
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    pub fn peer(&self) -> SocketAddr {
        self.peer
    }
}
