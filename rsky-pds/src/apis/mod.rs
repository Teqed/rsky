use crate::auth_verifier::AccessStandard;
use crate::handle;
use crate::handle::errors::ErrorKind;
use crate::pipethrough::{ProxyRequest, pipethrough_procedure, pipethrough_procedure_post};
use anyhow::{Error, Result};
use rocket::http::{ContentType, Header, Status};
use rocket::request::FromParam;
use rocket::serde::json::Json;
use rocket::{Data, Request, Responder, response};

#[derive(Responder)]
#[response(status = 200)]
pub struct ProxyResponder(Vec<u8>, Header<'static>, Header<'static>);

#[allow(dead_code)]
pub struct Nsid(String);

impl<'a> FromParam<'a> for Nsid {
    type Error = &'a str;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        // This is how we make sure we allowlist lexicons and what gets proxied
        if param.starts_with("app.bsky.") || param.starts_with("chat.bsky") {
            Ok(Nsid(param.to_string()))
        } else {
            Err(param)
        }
    }
}

// Lower ranks have higher presidence
#[tracing::instrument(skip_all)]
#[allow(unused_variables)]
#[rocket::get("/xrpc/<nsid>?<query..>", rank = 2)]
pub async fn bsky_api_get_forwarder(
    nsid: Nsid,
    query: Option<&str>,
    auth: AccessStandard,
    req: ProxyRequest<'_>,
) -> Result<ProxyResponder, ApiError> {
    let requester: Option<String> = match auth.access.credentials {
        None => None,
        Some(credentials) => credentials.did,
    };
    match pipethrough_procedure::<()>(&req, requester, None).await {
        Ok(res) => {
            let headers = res.headers.expect("Upstream responded without headers.");
            let content_length = match headers.get("content-length") {
                None => Header::new("content-length", res.buffer.len().to_string()),
                Some(val) => Header::new("content-length", val.to_string()),
            };
            let content_type = match headers.get("content-type") {
                None => Header::new("content-type", "octet-stream".to_string()),
                Some(val) => Header::new("Content-Type", val.to_string()),
            };
            Ok(ProxyResponder(res.buffer, content_length, content_type))
        }
        Err(error) => {
            tracing::error!("@LOG: ERROR: {error}");
            Err(ApiError::RuntimeError)
        }
    }
}

#[rocket::post("/xrpc/<_nsid>", data = "<body>", rank = 2)]
pub async fn bsky_api_post_forwarder(
    body: Data<'_>,
    _nsid: Nsid,
    auth: AccessStandard,
    req: ProxyRequest<'_>,
) -> Result<ProxyResponder, ApiError> {
    let requester: Option<String> = match auth.access.credentials {
        None => None,
        Some(credentials) => credentials.did,
    };

    let res = pipethrough_procedure_post(&req, requester, Some(body)).await?;
    let headers = res.headers.expect("Upstream responded without headers.");
    let content_length = match headers.get("content-length") {
        None => Header::new("content-length", res.buffer.len().to_string()),
        Some(val) => Header::new("content-length", val.to_string()),
    };
    let content_type = match headers.get("content-type") {
        None => Header::new("content-type", "application/octet-stream".to_string()),
        Some(val) => Header::new("Content-Type", val.to_string()),
    };
    Ok(ProxyResponder(res.buffer, content_length, content_type))
}

pub use rsky_pds_accountmanager::apis::ApiError;

pub mod app;
pub mod com;
