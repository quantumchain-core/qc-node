// src/net/sync_codec.rs
// QTC: wire codec for the state-sync request-response protocol.
//
// ⚠️ HIGHEST COMPILE-RISK FILE IN THIS SESSION. Everything else this
// session was checked by hand against the real code and a manual brace/
// structure pass. This file additionally depends on getting libp2p
// 0.53's `request_response::Codec` trait shape exactly right, which I
// could not verify by actually compiling (no cargo/network in this
// environment). Run `cargo build` on this specific file before trusting
// it — if `Codec`'s associated `Protocol` type bound doesn't match
// (AsRef<str> vs an older ProtocolName-style trait), the fix is
// localized to this file only; nothing else in the sync feature depends
// on the exact shape of this trait.
//
// Design choice: rather than depend on libp2p's optional cbor/json
// convenience Codecs (whose public availability varies across patch
// versions), this hand-rolls a simple length-prefixed bincode framing
// over the raw stream, using only `futures::io` (already a dependency)
// — fewer moving parts to get wrong.

use async_trait::async_trait;
use futures::prelude::*;
use libp2p::request_response::Codec;
use libp2p::StreamProtocol;
use std::io;

use crate::sync::{SyncRequest, SyncResponse};

/// Cap on a single sync message's wire size. Bounds memory use and turns
/// a malformed/hostile peer's oversized length prefix into a clean
/// rejection instead of an unbounded allocation.
const MAX_MESSAGE_SIZE: u32 = 8 * 1024 * 1024; // 8 MiB

#[derive(Debug, Clone, Default)]
pub struct SyncCodec;

async fn read_length_prefixed<T>(io: &mut T) -> io::Result<Vec<u8>>
where
    T: AsyncRead + Unpin + Send,
{
    let mut len_buf = [0u8; 4];
    io.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf);
    if len > MAX_MESSAGE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("sync message too large: {len} bytes (max {MAX_MESSAGE_SIZE})"),
        ));
    }
    let mut buf = vec![0u8; len as usize];
    io.read_exact(&mut buf).await?;
    Ok(buf)
}

async fn write_length_prefixed<T>(io: &mut T, bytes: Vec<u8>) -> io::Result<()>
where
    T: AsyncWrite + Unpin + Send,
{
    if bytes.len() > MAX_MESSAGE_SIZE as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("sync message too large to send: {} bytes (max {MAX_MESSAGE_SIZE})", bytes.len()),
        ));
    }
    let len = (bytes.len() as u32).to_be_bytes();
    io.write_all(&len).await?;
    io.write_all(&bytes).await?;
    io.flush().await?;
    Ok(())
}

#[async_trait]
impl Codec for SyncCodec {
    type Protocol = StreamProtocol;
    type Request = SyncRequest;
    type Response = SyncResponse;

    async fn read_request<T>(&mut self, _protocol: &Self::Protocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let bytes = read_length_prefixed(io).await?;
        bincode::deserialize(&bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(&mut self, _protocol: &Self::Protocol, io: &mut T) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let bytes = read_length_prefixed(io).await?;
        bincode::deserialize(&bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(&mut self, _protocol: &Self::Protocol, io: &mut T, req: Self::Request) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let bytes = bincode::serialize(&req).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        write_length_prefixed(io, bytes).await
    }

    async fn write_response<T>(&mut self, _protocol: &Self::Protocol, io: &mut T, res: Self::Response) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let bytes = bincode::serialize(&res).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        write_length_prefixed(io, bytes).await
    }
}
