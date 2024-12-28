use std::{io, time::Duration};

use mux::{MuxConfig, MuxError, StreamAccepter, StreamOpener, spawn_mux_no_reconnection};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    task::JoinSet,
};

use crate::protocol::codec::{
    CorruptedDataError, DecodeError, EncodeError, InsufficientBufferError, decode,
};

use super::codec::{AllowedDeserialize, AllowedSerialize, encode};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

pub async fn runner_establish<R, W>(
    mut r: R,
    mut w: W,
    runners: Vec<RunnerRegister>,
    mut mux_spawner: JoinSet<MuxError>,
) -> Result<StreamOpener, RunnerEstablishError>
where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
{
    let mut buf = [0; 1024];
    let req = RunnersRegisterRequest { runners };
    match encode(&mut w, &req, &mut buf).await {
        Ok(()) => (),
        Err(e) => return Err(RunnerEstablishError::Io(e.panic_or_into_io_error())),
    };
    let resp: RunnersRegisterResponse = match decode(&mut r, &mut buf).await {
        Ok(resp) => resp,
        Err(e) => return Err(RunnerEstablishError::Io(e.into_io_error())),
    };
    match resp {
        RunnersRegisterResponse::Ok => (),
        RunnersRegisterResponse::No => return Err(RunnerEstablishError::Rejected),
    }
    let config = MuxConfig {
        initiation: mux::Initiation::Client,
        heartbeat_interval: HEARTBEAT_INTERVAL,
    };
    let (opener, _accepter) = spawn_mux_no_reconnection(r, w, config, &mut mux_spawner);
    Ok(opener)
}
#[derive(Debug)]
pub enum RunnerEstablishError {
    Io(io::Error),
    Rejected,
}

pub async fn control_establish<R, W>(
    mut r: R,
    mut w: W,
    mut mux_spawner: JoinSet<MuxError>,
) -> Result<(StreamAccepter, Vec<RunnerRegister>), ControlEstablishError>
where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
{
    let mut buf = [0; 1024];
    let req: RunnersRegisterRequest = match decode(&mut r, &mut buf).await {
        Ok(req) => req,
        Err(e) => match e {
            DecodeError::InsufficientBufferError(InsufficientBufferError)
            | DecodeError::CorruptedData(CorruptedDataError) => {
                return Err(ControlEstablishError::Io(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Corrupted data",
                )));
            }
            DecodeError::Reader(e) => return Err(ControlEstablishError::Io(e)),
        },
    };
    let resp = RunnersRegisterResponse::Ok;
    match encode(&mut w, &resp, &mut buf).await {
        Ok(()) => (),
        Err(e) => match e {
            EncodeError::InsufficientBufferError(InsufficientBufferError) => panic!("{e:?}"),
            EncodeError::Writer(e) => return Err(ControlEstablishError::Io(e)),
        },
    };
    let config = MuxConfig {
        initiation: mux::Initiation::Server,
        heartbeat_interval: HEARTBEAT_INTERVAL,
    };
    let (_opener, accepter) = spawn_mux_no_reconnection(r, w, config, &mut mux_spawner);
    Ok((accepter, req.runners))
}
#[derive(Debug)]
pub enum ControlEstablishError {
    Io(io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunnersRegisterRequest {
    pub runners: Vec<RunnerRegister>,
}
impl AllowedSerialize for RunnersRegisterRequest {}
impl AllowedDeserialize for RunnersRegisterRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum RunnersRegisterResponse {
    Ok,
    No,
}
impl AllowedSerialize for RunnersRegisterResponse {}
impl AllowedDeserialize for RunnersRegisterResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerRegister {
    pub name: String,
}
