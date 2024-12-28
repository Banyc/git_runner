use std::io;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};

use super::codec::{AllowedDeserialize, AllowedSerialize, decode, encode};

pub async fn runner_send_service_event<W>(w: &mut W, event: &ServiceEvent) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let mut buf = [0; 1024];
    match encode(w, event, &mut buf).await {
        Ok(()) => (),
        Err(e) => return Err(e.panic_or_into_io_error()),
    };
    Ok(())
}

pub async fn control_recv_service_event<R>(r: &mut R) -> io::Result<ServiceEvent>
where
    R: AsyncRead + Unpin,
{
    let mut buf = [0; 1024];
    let event: ServiceEvent = match decode(r, &mut buf).await {
        Ok(event) => event,
        Err(e) => return Err(e.into_io_error()),
    };
    Ok(event)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceEvent {
    Exit(u32),
    Output(Output),
}
impl AllowedSerialize for ServiceEvent {}
impl AllowedDeserialize for ServiceEvent {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub ty: OutputType,
    pub size: u16,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputType {
    Stdout,
    Stderr,
}
