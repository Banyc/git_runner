use std::io;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::protocol::codec::{decode, encode};

use super::codec::{AllowedDeserialize, AllowedSerialize};

pub async fn control_create_service<R, W>(
    r: &mut R,
    w: &mut W,
    req: &CreateServiceRequest,
) -> Result<(), ControlCreateServiceError>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buf = [0; 1024];
    match encode(w, req, &mut buf).await {
        Ok(()) => (),
        Err(e) => return Err(ControlCreateServiceError::Io(e.panic_or_into_io_error())),
    };
    let resp: CreateServiceResponse = match decode(r, &mut buf).await {
        Ok(resp) => resp,
        Err(e) => return Err(ControlCreateServiceError::Io(e.into_io_error())),
    };
    match resp {
        CreateServiceResponse::Ok => (),
        CreateServiceResponse::No => return Err(ControlCreateServiceError::Rejected),
    }
    Ok(())
}
#[derive(Debug)]
pub enum ControlCreateServiceError {
    Io(io::Error),
    Rejected,
}

pub async fn runner_create_service_request<R>(r: &mut R) -> io::Result<CreateServiceRequest>
where
    R: AsyncRead + Unpin,
{
    let mut buf = [0; 1024];
    let req: CreateServiceRequest = match decode(r, &mut buf).await {
        Ok(req) => req,
        Err(e) => return Err(e.into_io_error()),
    };
    Ok(req)
}
pub async fn runner_create_service_respond<W>(
    w: &mut W,
    resp: CreateServiceResponse,
) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let mut buf = [0; 1024];
    match encode(w, &resp, &mut buf).await {
        Ok(()) => (),
        Err(e) => return Err(e.panic_or_into_io_error()),
    };
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceRequest {
    pub service_name: String,
    pub git_url: String,
    pub git_tag: String,
    pub exec_command: String,
    pub exec_args: Vec<String>,
}
impl AllowedSerialize for CreateServiceRequest {}
impl AllowedDeserialize for CreateServiceRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CreateServiceResponse {
    Ok,
    No,
}
impl AllowedSerialize for CreateServiceResponse {}
impl AllowedDeserialize for CreateServiceResponse {}
