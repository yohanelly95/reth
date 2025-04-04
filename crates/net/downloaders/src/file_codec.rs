//! Codec for reading raw block bodies from a file.

use crate::file_client::FileClientError;
use alloy_primitives::bytes::{Buf, BytesMut};
use alloy_rlp::{Decodable, Encodable};
use tokio_util::codec::{Decoder, Encoder};

/// Codec for reading raw block bodies from a file.
///
/// If using with [`FramedRead`](tokio_util::codec::FramedRead), the user should make sure the
/// framed reader has capacity for the entire block file. Otherwise, the decoder will return
/// [`InputTooShort`](alloy_rlp::Error::InputTooShort), because RLP headers can only be
/// decoded if the internal buffer is large enough to contain the entire block body.
///
/// Without ensuring the framed reader has capacity for the entire file, a block body is likely to
/// fall across two read buffers, the decoder will not be able to decode the header, which will
/// cause it to fail.
///
/// It's recommended to use [`with_capacity`](tokio_util::codec::FramedRead::with_capacity) to set
/// the capacity of the framed reader to the size of the file.
pub(crate) struct BlockFileCodec<B>(std::marker::PhantomData<B>);

impl<B> Default for BlockFileCodec<B> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<B: Decodable> Decoder for BlockFileCodec<B> {
    type Item = B;
    type Error = FileClientError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None)
        }

        let buf_slice = &mut src.as_ref();
        let body = B::decode(buf_slice).map_err(|err| FileClientError::Rlp(err, src.to_vec()))?;
        src.advance(src.len() - buf_slice.len());

        Ok(Some(body))
    }
}

impl<B: Encodable> Encoder<B> for BlockFileCodec<B> {
    type Error = FileClientError;

    fn encode(&mut self, item: B, dst: &mut BytesMut) -> Result<(), Self::Error> {
        item.encode(dst);
        Ok(())
    }
}
