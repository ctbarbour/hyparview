use crate::ProtocolMessage;
use bytes::{Buf, BytesMut};
use serde::Serialize;
use serde_cbor::ser::{Serializer};
use std::io::Cursor;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn write_frame(&mut self, message: ProtocolMessage) -> std::io::Result<()> {
        let mut buf = Vec::new();
        let mut serializer = Serializer::new(&mut buf);
        let _ = message.serialize(&mut serializer); // handle error here

        let length_prefix = buf.len() as u32;
        self.stream.write_u32(length_prefix).await?;
        self.stream.write_all(&buf).await?;

        self.stream.flush().await
    }

    pub async fn read_frame(
        &mut self,
    ) -> Result<Option<ProtocolMessage>, Box<dyn std::error::Error + Send + Sync>> {
        loop {
            let mut buf = Cursor::new(&self.buffer[..]);

            // If we have data in the read buffer
            if buf.has_remaining() {
                // Get the first byte in the read buffer as the length of the message
                let length = buf.get_u32() as usize;

                // check if we have a full message in the read buffer
                if buf.remaining() >= length {
                    // parse the message from the read buffer
                    let start_pos = buf.position() as usize;
                    let end_pos = start_pos + length;
                    println!(
                        "Received a frame start={:?},end={:?},length={:?},buf={:?}",
                        start_pos, end_pos, length, buf
                    );
                    let frame = serde_cbor::from_slice(&buf.get_ref()[start_pos..end_pos]).unwrap();

                    // discard the parsed data from the read buffer
                    self.buffer.advance(start_pos + length);

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
}
