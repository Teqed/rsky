use crate::auth_verifier::AccessStandard;
use crate::handle;
use crate::handle::errors::ErrorKind;
use crate::pipethrough::{ProxyRequest, pipethrough_procedure, pipethrough_procedure_post};
use anyhow::{Error, Result};
use rocket::http::{ContentType, Header, Status};
use rocket::request::FromParam;
use rocket::serde::json::Json;
use rocket::{Data, Request, Responder, response};

#[derive(Clone, Debug)]
pub enum ApiError {
    RuntimeError,
    InvalidLogin,
    AccountTakendown,
    InvalidRequest(String),
    ExpiredToken,
    InvalidToken,
    RecordNotFound,
    InvalidHandle,
    InvalidEmail,
    InvalidPassword,
    InvalidInviteCode,
    HandleNotAvailable,
    EmailNotAvailable,
    UnsupportedDomain,
    UnresolvableDid,
    IncompatibleDidDoc,
    WellKnownNotFound,
    AccountNotFound,
    BlobNotFound,
    BadRequest(String, String),
    AuthRequiredError(String),
}

#[derive(Serialize)]
pub struct ErrorBody {
    error: String,
    message: String,
}

impl<'r, 'o: 'r> ::rocket::response::Responder<'r, 'o> for ApiError {
    fn respond_to(self, __req: &'r Request<'_>) -> response::Result<'o> {
        match self {
            ApiError::RuntimeError => {
                let body = Json(ErrorBody {
                    error: "InternalServerError".to_string(),
                    message: "Something went wrong".to_string(),
                });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 500u16 });
                Ok(res)
            }
            ApiError::InvalidLogin => {
                let body = Json(ErrorBody {
                    error: "InvalidLogin".to_string(),
                    message: "Invalid identifier or password".to_string(),
                });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(::rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 400u16 });
                Ok(res)
            }
            ApiError::AccountTakendown => {
                let body = Json(ErrorBody {
                    error: "AccountTakendown".to_string(),
                    message: "Account has been taken down".to_string(),
                });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(::rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 400u16 });
                Ok(res)
            }
            ApiError::InvalidRequest(message) => {
                let body = Json(ErrorBody {
                    error: "InvalidRequest".to_string(),
                    message,
                });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 400u16 });
                Ok(res)
            }
            ApiError::ExpiredToken => {
                let body = Json(ErrorBody {
                    error: "ExpiredToken".to_string(),
                    message: "Token is expired".to_string(),
                });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 400u16 });
                Ok(res)
            }
            ApiError::InvalidToken => {
                let body = Json(ErrorBody {
                    error: "InvalidToken".to_string(),
                    message: "Token is invalid".to_string(),
                });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 400u16 });
                Ok(res)
            }
            ApiError::InvalidHandle => {
                let body = Json(ErrorBody {
                    error: "InvalidHandle".to_string(),
                    message: "Handle is invalid".to_string(),
                });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 400u16 });
                Ok(res)
            }
            ApiError::InvalidEmail => {
                let body = Json(ErrorBody {
                    error: "InvalidEmail".to_string(),
                    message: "Invalid email".to_string(),
                });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 400u16 });
                Ok(res)
            }
            ApiError::InvalidPassword => {
                let body = Json(ErrorBody {
                    error: "InvalidPassword".to_string(),
                    message: "Invalid Password".to_string(),
                });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 400u16 });
                Ok(res)
            }
            ApiError::InvalidInviteCode => {
                let body = Json(ErrorBody {
                    error: "InvalidInviteCode".to_string(),
                    message: "Invalid invite code".to_string(),
                });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 400u16 });
                Ok(res)
            }
            ApiError::HandleNotAvailable => {
                let body = Json(ErrorBody {
                    error: "HandleNotAvailable".to_string(),
                    message: "Handle not available".to_string(),
                });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 400u16 });
                Ok(res)
            }
            ApiError::EmailNotAvailable => {
                let body = Json(ErrorBody {
                    error: "EmailNotAvailable".to_string(),
                    message: "Email not available".to_string(),
                });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 400u16 });
                Ok(res)
            }
            ApiError::UnsupportedDomain => {
                let body = Json(ErrorBody {
                    error: "UnsupportedDomain".to_string(),
                    message: "Unsupported domain".to_string(),
                });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 400u16 });
                Ok(res)
            }
            ApiError::UnresolvableDid => {
                let body = Json(ErrorBody {
                    error: "UnresolvableDid".to_string(),
                    message: "Unresolved Did".to_string(),
                });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 400u16 });
                Ok(res)
            }
            ApiError::IncompatibleDidDoc => {
                let body = Json(ErrorBody {
                    error: "IncompatibleDidDoc".to_string(),
                    message: "IncompatibleDidDoc".to_string(),
                });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 400u16 });
                Ok(res)
            }
            ApiError::AccountNotFound => {
                let body = Json(ErrorBody {
                    error: "AccountNotFound".to_string(),
                    message: "Account could not be found".to_string(),
                });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 400u16 });
                Ok(res)
            }
            ApiError::BlobNotFound => {
                let body = Json(ErrorBody {
                    error: "BlobNotFound".to_string(),
                    message: "Blob could not be found".to_string(),
                });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 400u16 });
                Ok(res)
            }
            ApiError::WellKnownNotFound => {
                let body = Json(ErrorBody {
                    error: "WellKnownNotFound".to_string(),
                    message: "User not found".to_string(),
                });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(::rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 404u16 });
                Ok(res)
            }
            ApiError::BadRequest(error, message) => {
                let body = Json(ErrorBody { error, message });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 400u16 });
                Ok(res)
            }
            ApiError::AuthRequiredError(message) => {
                let body = Json(ErrorBody {
                    error: "AuthRequiredError".to_string(),
                    message,
                });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(::rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 401u16 });
                Ok(res)
            }
            ApiError::RecordNotFound => {
                let body = Json(ErrorBody {
                    error: "RecordNotFound".to_string(),
                    message: "Record could not be found".to_string(),
                });
                let mut res =
                    <Json<ErrorBody> as ::rocket::response::Responder>::respond_to(body, __req)?;
                res.set_header(ContentType(rocket::http::MediaType::const_new(
                    "application",
                    "json",
                    &[],
                )));
                res.set_status(Status { code: 404u16 });
                Ok(res)
            }
        }
    }
}

impl From<Error> for ApiError {
    fn from(_value: Error) -> Self {
        ApiError::RuntimeError
    }
}

impl From<handle::errors::Error> for ApiError {
    fn from(value: handle::errors::Error) -> Self {
        match value.kind {
            ErrorKind::InvalidHandle => ApiError::InvalidHandle,
            ErrorKind::HandleNotAvailable => ApiError::HandleNotAvailable,
            ErrorKind::UnsupportedDomain => ApiError::UnsupportedDomain,
            ErrorKind::InternalError => ApiError::RuntimeError,
        }
    }
}

pub mod com {
    pub mod atproto {
        pub mod server {
            use rand::{distributions::Alphanumeric, Rng};

            /// Formatted xxxxx-xxxxx
            pub fn get_random_token() -> String {
                let token: String = rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(50)
                    .map(char::from)
                    .collect();
                //Bluesky Client doesn't support 1,8,9,0 in the email verification tokens
                let allowed_token = token.replace(&['1', '8', '9', '0'][..], "");
                allowed_token[0..5].to_owned() + "-" + &allowed_token[5..10]
            }
        }
    }
}
