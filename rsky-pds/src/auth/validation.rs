use crate::auth::{
    access_output::AccessOutput,
    auth_scope::AuthScope,
    error::AuthError,
    jwt_payload::JwtPayload,
    service_jwt_opts::ServiceJwtOpts,
    validate_access_token_opts::ValidateAccessTokenOpts,
    validated_bearer::ValidatedBearer,
    verified_service_jwt::VerifiedServiceJwt,
    utils::{bearer_token_from_req},
};
use axum::http::HeaderMap;
use std::collections::HashSet;
use std::env;
use anyhow::{Result, bail};
use async_trait::async_trait;

// Placeholder for actual JWT and key types
// use jwt_simple::prelude::*;
// use crate::account_manager::helpers::auth::CustomClaimObj;

pub async fn access_check(
    headers: &HeaderMap,
    scopes: Vec<AuthScope>,
    opts: Option<ValidateAccessTokenOpts>,
) -> Result<AccessOutput, AuthError> {
    match validate_access_token(headers, scopes, opts).await {
        Ok(access) => Ok(access),
        Err(e) => Err(e),
    }
}

pub async fn validate_bearer_access_token(
    headers: &HeaderMap,
    scopes: Vec<AuthScope>,
) -> Result<AccessOutput, AuthError> {
    use std::env;
    let audience = env::var("PDS_SERVICE_DID").map_err(|_| AuthError::InternalServerError)?;
    // NOTE: VerificationOptions and audience checks would be passed to validate_bearer_token in a real implementation.
    let validated = validate_bearer_token(headers, scopes, Some(audience.clone())).await?;
    let is_privileged = matches!(validated.scope, AuthScope::Access | AuthScope::AppPassPrivileged);
    Ok(AccessOutput {
        credentials: Some(Credentials {
            r#type: "access".to_string(),
            did: Some(validated.did),
            scope: Some(validated.scope),
            audience: validated.audience.clone(),
            token_id: validated.payload.jti.clone(),
            aud: validated.payload.aud.as_ref().map(|a| format!("{:?}", a)),
            iss: None,
            is_privileged: Some(is_privileged),
        }),
        artifacts: Some(validated.token),
    })
}

pub async fn validate_bearer_token(
    headers: &HeaderMap,
    scopes: Vec<AuthScope>,
    verify_audience: Option<String>,
) -> Result<ValidatedBearer, AuthError> {
    use jwt_simple::prelude::*;
    use std::env;

    let token = bearer_token_from_req(headers).ok_or(AuthError::AccessDenied)?;
    // Load key from env
    let private_key_hex = env::var("PDS_JWT_KEY_K256_PRIVATE_KEY_HEX").map_err(|_| AuthError::InternalServerError)?;
    let private_key_bytes = hex::decode(private_key_hex).map_err(|_| AuthError::InternalServerError)?;
    let key_pair = ES256KKeyPair::from_bytes(&private_key_bytes).map_err(|_| AuthError::InternalServerError)?;

    // Verify JWT
    let claims = key_pair
        .public_key()
        .verify_token::<JwtClaims<serde_json::Value>>(&token, None)
        .map_err(|_| AuthError::BadJwt)?;

    // Extract custom claims (assume scope, sub, aud, etc. are present)
    let scope_str = claims.custom.get("scope").and_then(|v| v.as_str()).ok_or(AuthError::BadJwt)?;
    let scope = AuthScope::from_str(scope_str).map_err(|_| AuthError::InvalidScope)?;
    let sub = claims.subject.clone().ok_or(AuthError::BadJwt)?;
    let aud = claims.audiences.clone();
    let exp = claims.expires_at;
    let iat = claims.issued_at;
    let jti = claims.jwt_id.clone();

    // Audience check
    if let (Some(expected_aud), Some(audiences)) = (verify_audience, &aud) {
        let aud_str = match audiences {
            Audiences::AsString(aud_str) => aud_str,
            Audiences::AsArray(arr) => arr.get(0).ok_or(AuthError::BadJwt)?,
        };
        if aud_str != &expected_aud {
            return Err(AuthError::BadJwtAudience);
        }
    }

    // Scope check
    if !scopes.is_empty() && !scopes.contains(&scope) {
        return Err(AuthError::InvalidScope);
    }

    // Subject format check
    if !sub.starts_with("did:") {
        return Err(AuthError::BadJwt);
    }

    Ok(ValidatedBearer {
        did: sub,
        scope,
        token,
        payload: JwtPayload {
            scope,
            sub: claims.subject,
            aud: claims.audiences,
            exp,
            iat,
            jti,
        },
        audience: aud.and_then(|a| match a {
            Audiences::AsString(s) => Some(s),
            Audiences::AsArray(arr) => arr.get(0).cloned(),
        }),
    })
}

pub async fn validate_access_token(
    headers: &HeaderMap,
    scopes: Vec<AuthScope>,
    opts: Option<ValidateAccessTokenOpts>,
) -> Result<AccessOutput, AuthError> {
    use std::env;
    let audience = env::var("PDS_SERVICE_DID").map_err(|_| AuthError::InternalServerError)?;
    let validated = validate_bearer_token(headers, scopes, Some(audience.clone())).await?;

    // TODO: AccountManager logic for takedown/deactivated checks
    let ValidateAccessTokenOpts {
        check_takedown,
        check_deactivated,
    } = opts.unwrap_or(ValidateAccessTokenOpts {
        check_takedown: Some(false),
        check_deactivated: Some(false),
    });
    let _check_takedown = check_takedown.unwrap_or(false);
    let _check_deactivated = check_deactivated.unwrap_or(false);

    // TODO: If _check_takedown or _check_deactivated, check account status via account manager

    Ok(AccessOutput {
        credentials: Some(Credentials {
            r#type: "access".to_string(),
            did: Some(validated.did),
            scope: Some(validated.scope),
            audience: validated.audience.clone(),
            token_id: validated.payload.jti.clone(),
            aud: validated.payload.aud.as_ref().map(|a| format!("{:?}", a)),
            iss: None,
            is_privileged: None,
        }),
        artifacts: Some(validated.token),
    })
}

pub async fn verify_service_jwt(
    headers: &HeaderMap,
    opts: ServiceJwtOpts,
) -> Result<VerifiedServiceJwt, AuthError> {
    // TODO: Implement key resolver logic for service JWTs
    let token = bearer_token_from_req(headers).ok_or(AuthError::AccessDenied)?;

    // In a real implementation, resolve the key using opts.iss and verify the JWT
    // For now, just parse the JWT and extract aud/iss
    use jwt_simple::prelude::*;
    let dummy_key = ES256KKeyPair::generate();
    let claims = dummy_key
        .public_key()
        .verify_token::<JwtClaims<serde_json::Value>>(&token, None)
        .map_err(|_| AuthError::BadJwt)?;

    let aud = claims.audiences.as_ref().and_then(|a| match a {
        Audiences::AsString(s) => Some(s.clone()),
        Audiences::AsArray(arr) => arr.get(0).cloned(),
    }).unwrap_or_else(|| "audience".to_string());

    let iss = claims.issuer.unwrap_or_else(|| "issuer".to_string());

    Ok(VerifiedServiceJwt {
        aud,
        iss,
    })
}

pub async fn verify_jwt(
    jwt: String,
    jwt_key: ES256KKeyPair,
    // _verify_options: Option<VerificationOptions>,
) -> Result<JwtPayload, AuthError> {
    use jwt_simple::prelude::*;
    let claims = jwt_key
        .public_key()
        .verify_token::<JwtClaims<serde_json::Value>>(&jwt, None)
        .map_err(|_| AuthError::BadJwt)?;

    let scope_str = claims.custom.get("scope").and_then(|v| v.as_str()).ok_or(AuthError::BadJwt)?;
    let scope = AuthScope::from_str(scope_str).map_err(|_| AuthError::InvalidScope)?;

    Ok(JwtPayload {
        scope,
        sub: claims.subject,
        aud: claims.audiences,
        exp: claims.expires_at,
        iat: claims.issued_at,
        jti: claims.jwt_id,
    })
}
