use std::io;

use serde::{Serialize, de::DeserializeOwned};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub trait AllowedSerialize: Serialize {}
pub trait AllowedDeserialize: DeserializeOwned {}

pub async fn encode<W, T>(wtr: &mut W, value: &T, buf: &mut [u8]) -> Result<(), EncodeError>
where
    W: AsyncWrite + Unpin,
    T: AllowedSerialize,
{
    let mut buf_wtr = io::Cursor::new(&mut buf[..]);
    bincode::serialize_into(&mut buf_wtr, value)
        .map_err(|_| EncodeError::InsufficientBufferError(InsufficientBufferError))?;
    let size = buf_wtr.position();
    let size_usize = usize::try_from(size).unwrap();
    let f = async {
        wtr.write_all(&size.to_be_bytes()[..]).await?;
        wtr.write_all(&buf[..size_usize]).await?;
        Ok(())
    };
    f.await.map_err(EncodeError::Writer)?;
    Ok(())
}
#[derive(Debug)]
pub enum EncodeError {
    InsufficientBufferError(InsufficientBufferError),
    Writer(io::Error),
}
impl EncodeError {
    pub fn panic_or_into_io_error(self) -> io::Error {
        match self {
            EncodeError::InsufficientBufferError(InsufficientBufferError) => {
                panic!("{self:?}")
            }
            EncodeError::Writer(e) => e,
        }
    }
}

pub async fn decode<R, T>(rdr: &mut R, buf: &mut [u8]) -> Result<T, DecodeError>
where
    R: AsyncRead + Unpin,
    T: AllowedDeserialize,
{
    let f = async {
        let mut size_buf = 0_u64.to_be_bytes();
        rdr.read_exact(&mut size_buf)
            .await
            .map_err(DecodeError::Reader)?;
        let size = u64::from_be_bytes(size_buf);
        let size_usize =
            usize::try_from(size).map_err(|_| DecodeError::CorruptedData(CorruptedDataError))?;
        if buf.len() < size_usize {
            return Err(DecodeError::InsufficientBufferError(
                InsufficientBufferError,
            ));
        }
        let buf = &mut buf[..size_usize];
        rdr.read_exact(buf).await.map_err(DecodeError::Reader)?;
        Ok(&buf[..])
    };
    let buf = f.await?;
    let value =
        bincode::deserialize(buf).map_err(|_| DecodeError::CorruptedData(CorruptedDataError))?;
    Ok(value)
}
#[derive(Debug)]
pub enum DecodeError {
    InsufficientBufferError(InsufficientBufferError),
    CorruptedData(CorruptedDataError),
    Reader(io::Error),
}
impl DecodeError {
    pub fn into_io_error(self) -> io::Error {
        match self {
            DecodeError::InsufficientBufferError(InsufficientBufferError)
            | DecodeError::CorruptedData(CorruptedDataError) => {
                io::Error::new(io::ErrorKind::InvalidData, "Corrupted data")
            }
            DecodeError::Reader(e) => e,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InsufficientBufferError;
#[derive(Debug, Clone)]
pub struct CorruptedDataError;
