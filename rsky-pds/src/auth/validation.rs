use crate::account_manager::helpers::account::{ActorAccount, AvailabilityFlags};
use crate::account_manager::helpers::auth::CustomClaimObj;
use crate::auth::{
    error::AuthError,
    types::{
        AccessOutput, Credentials, JwtPayload, ServiceJwtOpts, ValidateAccessTokenOpts,
        VerifiedServiceJwt,
    },
    utils::bearer_token_from_req,
    AuthScope, ValidatedBearer,
};
use crate::AccountManager;
use crate::AppState;
use anyhow::{bail, Result};
use axum::http::HeaderMap;
use jwt_simple::prelude::*;
use secp256k1::{Keypair, Secp256k1, SecretKey};
use std::collections::HashSet;
use std::env;
use std::sync::Arc;

pub async fn access_check(
    headers: &HeaderMap,
    scopes: Vec<AuthScope>,
    opts: Option<ValidateAccessTokenOpts>,
    account_manager: &AccountManager,
) -> Result<AccessOutput, AuthError> {
    validate_access_token(headers, scopes, opts, account_manager).await
}

pub async fn validate_bearer_access_token(
    headers: &HeaderMap,
    scopes: Vec<AuthScope>,
) -> Result<AccessOutput, AuthError> {
    let mut options = VerificationOptions::default();
    options.allowed_audiences = Some(HashSet::from_strings(&[
        env::var("PDS_SERVICE_DID").unwrap()
    ]));
    let ValidatedBearer {
        did,
        scope,
        token,
        audience,
        ..
    } = validate_bearer_token(headers, scopes, Some(options)).await?;
    let is_privileged = vec![AuthScope::Access, AuthScope::AppPassPrivileged].contains(&scope);
    Ok(AccessOutput {
        credentials: Some(Credentials {
            r#type: "access".to_string(),
            did: Some(did),
            scope: Some(scope),
            audience,
            token_id: None,
            aud: None,
            iss: None,
            is_privileged: Some(is_privileged),
        }),
        artifacts: Some(token),
    })
}

pub async fn validate_bearer_token(
    headers: &HeaderMap,
    scopes: Vec<AuthScope>,
    verify_options: Option<VerificationOptions>,
) -> Result<ValidatedBearer, AuthError> {
    let token = bearer_token_from_req(headers)?;
    if let Some(token) = token {
        let secp = Secp256k1::new();
        let private_key = env::var("PDS_JWT_KEY_K256_PRIVATE_KEY_HEX").unwrap();
        let secret_key =
            SecretKey::from_slice(&hex::decode(private_key.as_bytes()).unwrap()).unwrap();
        let jwt_key = Keypair::from_secret_key(&secp, &secret_key);
        let payload = verify_jwt(token.clone(), jwt_key, verify_options).await?;
        let JwtPayload {
            sub, aud, scope, ..
        } = payload.clone();
        let sub = sub.ok_or(AuthError::BadJwt)?;
        let aud = aud.ok_or(AuthError::BadJwtAudience)?;
        if !sub.starts_with("did:") {
            bail!("Malformed token")
        }
        if let Audiences::AsString(aud) = aud {
            if !aud.starts_with("did:") {
                bail!("Malformed token")
            }
            if !scopes.is_empty() && !scopes.contains(&scope) {
                bail!("Bad token scope")
            }
            Ok(ValidatedBearer {
                did: sub,
                scope,
                audience: Some(aud),
                token,
                payload,
            })
        } else {
            bail!("Malformed token")
        }
    } else {
        bail!("AuthMissing")
    }
}

pub async fn validate_access_token(
    headers: &HeaderMap,
    scopes: Vec<AuthScope>,
    opts: Option<ValidateAccessTokenOpts>,
    account_manager: &AccountManager,
) -> Result<AccessOutput, AuthError> {
    let mut options = VerificationOptions::default();
    options.allowed_audiences = Some(HashSet::from_strings(&[
        env::var("PDS_SERVICE_DID").unwrap()
    ]));
    let ValidatedBearer {
        did,
        scope,
        token,
        audience,
        ..
    } = validate_bearer_token(headers, scopes, Some(options)).await?;
    let ValidateAccessTokenOpts {
        check_takedown,
        check_deactivated,
    } = opts.unwrap_or_else(|| ValidateAccessTokenOpts {
        check_takedown: Some(false),
        check_deactivated: Some(false),
    });
    let check_takedown = check_takedown.unwrap_or(false);
    let check_deactivated = check_deactivated.unwrap_or(false);

    if check_takedown || check_deactivated {
        let found: ActorAccount = match account_manager
            .get_account(
                &did,
                Some(AvailabilityFlags {
                    include_deactivated: Some(true),
                    include_taken_down: Some(true),
                }),
            )
            .await
        {
            Ok(Some(found)) => found,
            _ => return Err(AuthError::AccountNotFound),
        };
        if check_takedown && found.takedown_ref.is_some() {
            return Err(AuthError::AccountTakedown);
        }
        if check_deactivated && found.deactivated_at.is_some() {
            return Err(AuthError::AccountDeactivated);
        }
    }
    Ok(AccessOutput {
        credentials: Some(Credentials {
            r#type: "access".to_string(),
            did: Some(did),
            scope: Some(scope),
            audience,
            token_id: None,
            aud: None,
            iss: None,
            is_privileged: None,
        }),
        artifacts: Some(token),
    })
}

pub async fn verify_service_jwt(
    headers: &HeaderMap,
    opts: ServiceJwtOpts,
    // You may need to pass additional state here if needed for DID resolution
) -> Result<VerifiedServiceJwt, AuthError> {
    let get_signing_key = |iss: String, force_refresh: bool| -> Result<String> {
        match &opts.iss {
            Some(opts_iss) if opts_iss.contains(&iss) => bail!("UntrustedIss: Untrusted issuer"),
            _ => (),
        }
        // You will need to implement DID resolution logic here, using your app's state if needed.
        bail!("DID resolution not implemented in Axum context")
    };

    match bearer_token_from_req(headers)? {
        None => bail!("MissingJwt: missing jwt"),
        Some(jwt_str) => {
            // You will need to implement or adapt verify_service_jwt_server for Axum/state
            // let payload: ServiceJwtPayload =
            //     verify_service_jwt_server(jwt_str, opts.aud, get_signing_key).await?;
            // Ok(VerifiedServiceJwt {
            //     iss: payload.iss,
            //     aud: payload.aud,
            // })
            bail!("verify_service_jwt_server not implemented in Axum context")
        }
    }
}

pub async fn verify_jwt(
    jwt: String,
    jwt_key: Keypair,
    verify_options: Option<VerificationOptions>,
) -> Result<JwtPayload, AuthError> {
    let key = ES256kKeyPair::from_bytes(jwt_key.secret_bytes().as_slice())?;
    let public_key = key.public_key();
    let claims = public_key.verify_token::<CustomClaimObj>(&jwt, verify_options)?;

    Ok(JwtPayload {
        scope: AuthScope::from_str(&claims.custom.scope)?,
        sub: claims.subject,
        aud: claims.audiences,
        exp: claims.expires_at,
        iat: claims.issued_at,
        jti: claims.jwt_id,
    })
}

// You may need to define or import CustomClaimObj and ServiceJwtPayload for your context.
