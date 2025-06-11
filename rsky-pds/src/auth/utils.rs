use crate::auth::{AccessOutput, BasicAuth};
use axum::http::HeaderMap;
use base64::{engine::general_purpose::STANDARD as base64pad, Engine as _};
use std::str;

pub fn is_user_or_admin(auth: AccessOutput, did: &String) -> bool {
    match auth.credentials {
        Some(credentials) if credentials.did == Some("admin_token".to_string()) => true,
        Some(credentials) => credentials.did == Some(did.to_string()),
        None => false,
    }
}

const BEARER: &str = "Bearer ";
const BASIC: &str = "Basic ";

pub fn is_bearer_token(headers: &HeaderMap) -> bool {
    match headers.get("Authorization").or_else(|| headers.get("authorization")) {
        None => false,
        Some(auth_header) => auth_header.to_str().map_or(false, |s| s.starts_with(BEARER)),
    }
}

pub fn is_basic_token(headers: &HeaderMap) -> bool {
    match headers.get("Authorization").or_else(|| headers.get("authorization")) {
        None => false,
        Some(auth_header) => auth_header.to_str().map_or(false, |s| s.starts_with(BASIC)),
    }
}

pub fn bearer_token_from_req(headers: &HeaderMap) -> Result<Option<String>, crate::auth::error::AuthError> {
    match headers.get("authorization").or_else(|| headers.get("Authorization")) {
        Some(header) => {
            let header_str = header.to_str().map_err(|_| crate::auth::error::AuthError::BadJwt)?;
            if !header_str.starts_with(BEARER) {
                Ok(None)
            } else {
                let slice = &header_str[BEARER.len()..];
                Ok(Some(slice.to_string()))
            }
        }
        None => Ok(None),
    }
}

pub fn parse_basic_auth(token: &str) -> Option<BasicAuth> {
    if !token.starts_with(BASIC) {
        return None;
    }

    let b64 = &token[BASIC.len()..];
    let decoded: Vec<u8> = match base64pad.decode(b64) {
        Err(_) => return None,
        Ok(decoded) => decoded,
    };
    let parsed_str: &str = match str::from_utf8(&decoded) {
        Err(_) => return None,
        Ok(res) => res,
    };
    let parsed_parts = parsed_str.split(":").collect::<Vec<&str>>();

    match (parsed_parts.get(0), parsed_parts.get(1)) {
        (Some(username), Some(password)) => Some(BasicAuth {
            username: username.to_string(),
            password: password.to_string(),
        }),
        _ => None,
    }
}
