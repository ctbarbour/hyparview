use crate::messages::*;
use bytes::{Buf, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug)]
pub struct HyParViewCodec;

impl HyParViewCodec {
    pub fn new() -> HyParViewCodec {
        HyParViewCodec {}
    }
}

impl Encoder<Box<dyn Message + Send>> for HyParViewCodec {
    type Error = std::io::Error;

    fn encode(
        &mut self,
        item: Box<dyn Message + Send>,
        buffer: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        item.encode(buffer)
    }
}

impl Decoder for HyParViewCodec {
    type Item = Box<dyn Message + Send>;
    type Error = std::io::Error;

    fn decode(&mut self, buffer: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if buffer.len() < 1 {
            return Ok(None);
        }

        let message_type = buffer.get_u8();

        let mut message: Box<dyn Message + Send> = match message_type {
            1 => Box::new(JoinMessage {}),
            2 => Box::new(ShuffleMessage::default()),
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unknown message type: {}", message_type),
                ))
            }
        };

        match message.decode(buffer) {
            Ok(_) => Ok(Some(message)),
            Err(err) => Err(err),
        }
    }
}
