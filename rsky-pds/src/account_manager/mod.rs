use crate::account_manager::helpers::account::{
    AccountStatus, ActorAccount, AvailabilityFlags, GetAccountAdminStatusOutput,
};
use crate::account_manager::helpers::auth::{
    AuthHelperError, CreateTokensOpts, RefreshGracePeriodOpts,
};
use crate::account_manager::helpers::invite::CodeDetail;
use crate::account_manager::helpers::password::UpdateUserPasswordOpts;
use crate::account_manager::helpers::token::FindByQbOpts;
use crate::account_manager::helpers::{authorization_request, device_account, repo};
use crate::account_manager::helpers::{token, used_refresh_token};
use crate::actor_store::ActorStorage;
use crate::auth::auth_scope::AuthScope;
use crate::models::models::pds::EmailTokenPurpose;
use anyhow::{bail, Result};
use chrono::offset::Utc as UtcOffset;
use chrono::DateTime;
use diesel::*;
use futures::try_join;
use helpers::{account, auth, device, email_token, invite, password};
use lexicon_cid::Cid;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use rsky_common;
use rsky_common::time::{from_micros_to_str, from_str_to_micros, HOUR};
use rsky_common::RFC3339_VARIANT;
use rsky_lexicon::com::atproto::admin::StatusAttr;
use rsky_lexicon::com::atproto::server::{AccountCodes, CreateAppPasswordOutput};
use rsky_oauth::jwk::Audience;
use rsky_oauth::oauth_provider::account::account_store::{
    AccountInfo, AccountStore, SignInCredentials,
};
use rsky_oauth::oauth_provider::client::client_store::ClientStore;
use rsky_oauth::oauth_provider::device::device_data::DeviceData;
use rsky_oauth::oauth_provider::device::device_id::DeviceId;
use rsky_oauth::oauth_provider::device::device_store::{DeviceStore, PartialDeviceData};
use rsky_oauth::oauth_provider::errors::OAuthError;
use rsky_oauth::oauth_provider::oidc::sub::Sub;
use rsky_oauth::oauth_provider::request::code::Code;
use rsky_oauth::oauth_provider::request::request_data::RequestData;
use rsky_oauth::oauth_provider::request::request_id::RequestId;
use rsky_oauth::oauth_provider::request::request_store::{
    FoundRequestResult, RequestStore, UpdateRequestData,
};
use rsky_oauth::oauth_provider::token::refresh_token::RefreshToken;
use rsky_oauth::oauth_provider::token::token_data::TokenData;
use rsky_oauth::oauth_provider::token::token_id::TokenId;
use rsky_oauth::oauth_provider::token::token_store::{NewTokenData, TokenInfo, TokenStore};
use rsky_oauth::oauth_types::{OAuthClientId, OAuthClientMetadata};
use secp256k1::{Keypair, Secp256k1, SecretKey};
use std::collections::BTreeMap;
use std::env;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

pub mod helpers;

/// Helps with readability when calling create_account()
pub struct CreateAccountOpts {
    pub did: String,
    pub handle: String,
    pub email: Option<String>,
    pub password: Option<String>,
    pub repo_cid: Cid,
    pub repo_rev: String,
    pub invite_code: Option<String>,
    pub deactivated: Option<bool>,
}

pub struct ConfirmEmailOpts<'em> {
    pub did: &'em String,
    pub token: &'em String,
}

pub struct ResetPasswordOpts {
    pub password: String,
    pub token: String,
}

pub struct UpdateAccountPasswordOpts {
    pub did: String,
    pub password: String,
}

pub struct UpdateEmailOpts {
    pub did: String,
    pub email: String,
}

pub struct DisableInviteCodesOpts {
    pub codes: Vec<String>,
    pub accounts: Vec<String>,
}

#[derive(Clone)]
pub struct AccountManager {
    pub db: deadpool_diesel::Pool<
        deadpool_diesel::Manager<SqliteConnection>,
        deadpool_diesel::sqlite::Object,
    >,
}
impl std::fmt::Debug for AccountManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccountManager").finish()
    }
}

pub type AccountManagerCreator = Box<
    dyn Fn(
            deadpool_diesel::Pool<
                deadpool_diesel::Manager<SqliteConnection>,
                deadpool_diesel::sqlite::Object,
            >,
        ) -> AccountManager
        + Send
        + Sync,
>;

impl AccountManager {
    pub const fn new(
        db: deadpool_diesel::Pool<
            deadpool_diesel::Manager<SqliteConnection>,
            deadpool_diesel::sqlite::Object,
        >,
    ) -> Self {
        Self { db }
    }

    pub fn creator() -> Arc<RwLock<AccountManagerCreator>> {
        Arc::new(RwLock::new(Box::new(
            move |db: deadpool_diesel::Pool<
                deadpool_diesel::Manager<SqliteConnection>,
                deadpool_diesel::sqlite::Object,
            >|
                  -> Self { Self::new(db) },
        )))
    }

    pub async fn get_account(
        &self,
        handle_or_did: &str,
        flags: Option<AvailabilityFlags>,
    ) -> Result<Option<ActorAccount>> {
        account::get_account(handle_or_did, flags, &self.db).await
    }

    pub async fn get_account_by_email(
        &self,
        email: &str,
        flags: Option<AvailabilityFlags>,
    ) -> Result<Option<ActorAccount>> {
        account::get_account_by_email(email, flags, &self.db).await
    }

    pub async fn is_account_activated(&self, did: &str) -> Result<bool> {
        let account = self
            .get_account(
                did,
                Some(AvailabilityFlags {
                    include_taken_down: None,
                    include_deactivated: Some(true),
                }),
            )
            .await?;
        if let Some(account) = account {
            Ok(account.deactivated_at.is_none())
        } else {
            Ok(false)
        }
    }

    pub async fn get_did_for_actor(
        &self,
        handle_or_did: &str,
        flags: Option<AvailabilityFlags>,
    ) -> Result<Option<String>> {
        match self.get_account(handle_or_did, flags).await {
            Ok(Some(got)) => Ok(Some(got.did)),
            _ => Ok(None),
        }
    }

    pub async fn create_account(
        &self,
        opts: CreateAccountOpts,
        actor_pools: &mut std::collections::HashMap<String, ActorStorage>,
    ) -> Result<(String, String)> {
        let CreateAccountOpts {
            did,
            handle,
            email,
            password,
            repo_cid,
            repo_rev,
            invite_code,
            deactivated,
        } = opts;
        let password_encrypted: Option<String> = match password {
            Some(password) => Some(password::gen_salt_and_hash(password)?),
            None => None,
        };
        // Should be a global var so this only happens once
        let secp = Secp256k1::new();
        let private_key = env::var("PDS_JWT_KEY_K256_PRIVATE_KEY_HEX")?;
        let secret_key =
            SecretKey::from_slice(&Result::unwrap(hex::decode(private_key.as_bytes())))?;
        let jwt_key = Keypair::from_secret_key(&secp, &secret_key);
        let (access_jwt, refresh_jwt) = auth::create_tokens(CreateTokensOpts {
            did: did.clone(),
            jwt_key,
            service_did: env::var("PDS_SERVICE_DID").expect("PDS_SERVICE_DID not set"),
            scope: Some(AuthScope::Access),
            jti: None,
            expires_in: None,
        })?;
        let refresh_payload = auth::decode_refresh_token(refresh_jwt.clone(), jwt_key)?;
        let now = rsky_common::now();

        if let Some(invite_code) = invite_code.clone() {
            invite::ensure_invite_is_available(invite_code, &self.db).await?;
        }
        account::register_actor(did.clone(), handle, deactivated, &self.db).await?;
        if let (Some(email), Some(password_encrypted)) = (email, password_encrypted) {
            account::register_account(did.clone(), email, password_encrypted, &self.db).await?;
        }
        invite::record_invite_use(did.clone(), invite_code, now, &self.db).await?;
        auth::store_refresh_token(refresh_payload, None, &self.db).await?;

        let did_path = did
            .strip_prefix("did:plc:")
            .ok_or_else(|| anyhow::anyhow!("Invalid DID"))?;
        let repo_path = format!("sqlite://data/repo/{}.db", did_path);
        let actor_repo_pool =
            crate::db::establish_pool(repo_path.as_str()).expect("Failed to establish pool");
        let blob_path = std::path::Path::new("data/blob").to_path_buf();
        let actor_pool = ActorStorage {
            repo: actor_repo_pool,
            blob: blob_path.clone(),
        };
        let blob_path = blob_path.join(did_path);
        tokio::fs::create_dir_all(&blob_path)
            .await
            .map_err(|_| anyhow::anyhow!("Failed to create blob path"))?;
        drop(
            actor_pools
                .insert(did.clone(), actor_pool)
                .expect("Failed to insert actor pools"),
        );
        let db = actor_pools
            .get(&did)
            .ok_or_else(|| anyhow::anyhow!("Actor not found"))?
            .repo
            .clone();
        repo::update_root(did, repo_cid, repo_rev, &db).await?;
        Ok((access_jwt, refresh_jwt))
    }

    pub async fn get_account_admin_status(
        &self,
        did: &str,
    ) -> Result<Option<GetAccountAdminStatusOutput>> {
        account::get_account_admin_status(did, &self.db).await
    }

    pub async fn update_repo_root(
        &self,
        did: String,
        cid: Cid,
        rev: String,
        actor_pools: &std::collections::HashMap<String, ActorStorage>,
    ) -> Result<()> {
        let db = actor_pools
            .get(&did)
            .ok_or_else(|| anyhow::anyhow!("Actor not found"))?
            .repo
            .clone();
        repo::update_root(did, cid, rev, &db).await
    }

    pub async fn delete_account(
        &self,
        did: &str,
        actor_pools: &std::collections::HashMap<String, ActorStorage>,
    ) -> Result<()> {
        let db = actor_pools
            .get(did)
            .ok_or_else(|| anyhow::anyhow!("Actor not found"))?
            .repo
            .clone();
        account::delete_account(did, &self.db, &db).await
    }

    pub async fn takedown_account(&self, did: &str, takedown: StatusAttr) -> Result<()> {
        (_, _) = try_join!(
            account::update_account_takedown_status(did, takedown, &self.db),
            auth::revoke_refresh_tokens_by_did(did, &self.db)
        )?;
        Ok(())
    }

    // @NOTE should always be paired with a sequenceHandle().
    pub async fn update_handle(&self, did: &str, handle: &str) -> Result<()> {
        account::update_handle(did, handle, &self.db).await
    }

    pub async fn deactivate_account(&self, did: &str, delete_after: Option<String>) -> Result<()> {
        account::deactivate_account(did, delete_after, &self.db).await
    }

    pub async fn activate_account(&self, did: &str) -> Result<()> {
        account::activate_account(did, &self.db).await
    }

    pub async fn get_account_status(&self, handle_or_did: &str) -> Result<AccountStatus> {
        let got = account::get_account(
            handle_or_did,
            Some(AvailabilityFlags {
                include_deactivated: Some(true),
                include_taken_down: Some(true),
            }),
            &self.db,
        )
        .await?;
        let res = account::format_account_status(got);
        match res.active {
            true => Ok(AccountStatus::Active),
            false => Ok(res.status.expect("Account status not properly formatted.")),
        }
    }

    // Auth
    // ----------
    pub async fn create_session(
        &self,
        did: String,
        app_password_name: Option<String>,
    ) -> Result<(String, String)> {
        let secp = Secp256k1::new();
        let private_key = env::var("PDS_JWT_KEY_K256_PRIVATE_KEY_HEX")?;
        let secret_key = SecretKey::from_slice(&hex::decode(private_key.as_bytes())?)?;
        let jwt_key = Keypair::from_secret_key(&secp, &secret_key);
        let scope = if app_password_name.is_none() {
            AuthScope::Access
        } else {
            AuthScope::AppPass
        };
        let (access_jwt, refresh_jwt) = auth::create_tokens(CreateTokensOpts {
            did,
            jwt_key,
            service_did: env::var("PDS_SERVICE_DID").expect("PDS_SERVICE_DID not set"),
            scope: Some(scope),
            jti: None,
            expires_in: None,
        })?;
        let refresh_payload = auth::decode_refresh_token(refresh_jwt.clone(), jwt_key)?;
        auth::store_refresh_token(refresh_payload, app_password_name, &self.db).await?;
        Ok((access_jwt, refresh_jwt))
    }

    pub async fn rotate_refresh_token(&self, id: &String) -> Result<Option<(String, String)>> {
        let token = auth::get_refresh_token(id, &self.db).await?;
        if let Some(token) = token {
            let system_time = SystemTime::now();
            let dt: DateTime<UtcOffset> = system_time.into();
            let now = format!("{}", dt.format(RFC3339_VARIANT));

            // take the chance to tidy all of a user's expired tokens
            // does not need to be transactional since this is just best-effort
            auth::delete_expired_refresh_tokens(&token.did, now, &self.db).await?;

            // Shorten the refresh token lifespan down from its
            // original expiration time to its revocation grace period.
            let prev_expires_at = from_str_to_micros(&token.expires_at);

            const REFRESH_GRACE_MS: i32 = 2 * HOUR;
            let grace_expires_at = dt.timestamp_micros() + REFRESH_GRACE_MS as i64;

            let expires_at = if grace_expires_at < prev_expires_at {
                grace_expires_at
            } else {
                prev_expires_at
            };

            if expires_at <= dt.timestamp_micros() {
                return Ok(None);
            }

            // Determine the next refresh token id: upon refresh token
            // reuse you always receive a refresh token with the same id.
            let next_id = token.next_id.unwrap_or_else(auth::get_refresh_token_id);

            let secp = Secp256k1::new();
            let private_key = env::var("PDS_JWT_KEY_K256_PRIVATE_KEY_HEX")
                .expect("PDS_JWT_KEY_K256_PRIVATE_KEY_HEX not set");
            let secret_key =
                SecretKey::from_slice(&hex::decode(private_key.as_bytes()).expect("Invalid key"))?;
            let jwt_key = Keypair::from_secret_key(&secp, &secret_key);

            let (access_jwt, refresh_jwt) = auth::create_tokens(CreateTokensOpts {
                did: token.did,
                jwt_key,
                service_did: env::var("PDS_SERVICE_DID").expect("PDS_SERVICE_DID not set"),
                scope: Some(if token.app_password_name.is_none() {
                    AuthScope::Access
                } else {
                    AuthScope::AppPass
                }),
                jti: Some(next_id.clone()),
                expires_in: None,
            })?;
            let refresh_payload = auth::decode_refresh_token(refresh_jwt.clone(), jwt_key)?;
            match try_join!(
                auth::add_refresh_grace_period(
                    RefreshGracePeriodOpts {
                        id: id.clone(),
                        expires_at: from_micros_to_str(expires_at),
                        next_id
                    },
                    &self.db
                ),
                auth::store_refresh_token(refresh_payload, token.app_password_name, &self.db)
            ) {
                Ok(_) => Ok(Some((access_jwt, refresh_jwt))),
                Err(e) => match e.downcast_ref() {
                    Some(AuthHelperError::ConcurrentRefresh) => {
                        Box::pin(self.rotate_refresh_token(id)).await
                    }
                    _ => Err(e),
                },
            }
        } else {
            Ok(None)
        }
    }

    pub async fn revoke_refresh_token(&self, id: String) -> Result<bool> {
        auth::revoke_refresh_token(id, &self.db).await
    }

    // Login
    // ----------
    pub async fn login(
        &self,
        identifier: String,
        password: String,
    ) -> Result<(ActorAccount, Option<String>, bool)> {
        let identifier_normalized = identifier.to_lowercase();
        let user = if identifier_normalized.contains("@") {
            self.get_account_by_email(
                identifier_normalized.as_str(),
                Some(AvailabilityFlags {
                    include_taken_down: Some(true),
                    include_deactivated: Some(true),
                }),
            )
            .await?
        } else {
            self.get_account(
                identifier_normalized.as_str(),
                Some(AvailabilityFlags {
                    include_taken_down: Some(true),
                    include_deactivated: Some(true),
                }),
            )
            .await?
        };

        let user = match user {
            None => {
                bail!("Invalid identifier or password");
            }
            Some(user) => user,
        };
        let is_soft_deleted = user.takedown_ref.is_some();
        let mut app_password: Option<String> = None;
        let valid_account_pass = self
            .verify_account_password(user.did.as_str(), &password)
            .await?;
        if !valid_account_pass {
            // takendown/suspended accounts cannot login with app password
            if is_soft_deleted {
                bail!("Invalid identifier or password");
            }
            app_password = self
                .verify_app_password(user.did.as_str(), password.as_str())
                .await?;
            if app_password.is_none() {
                bail!("Invalid identifier or password");
            }
        }
        Ok((user, app_password, is_soft_deleted))
    }

    // Invites
    // ----------

    pub async fn create_invite_codes(
        &self,
        to_create: Vec<AccountCodes>,
        use_count: i32,
    ) -> Result<()> {
        invite::create_invite_codes(to_create, use_count, &self.db).await
    }

    pub async fn create_account_invite_codes(
        &self,
        for_account: &str,
        codes: Vec<String>,
        expected_total: usize,
        disabled: bool,
    ) -> Result<Vec<CodeDetail>> {
        invite::create_account_invite_codes(for_account, codes, expected_total, disabled, &self.db)
            .await
    }

    pub async fn get_account_invite_codes(&self, did: &str) -> Result<Vec<CodeDetail>> {
        invite::get_account_invite_codes(did, &self.db).await
    }

    pub async fn get_invited_by_for_accounts(
        &self,
        dids: Vec<String>,
    ) -> Result<BTreeMap<String, CodeDetail>> {
        invite::get_invited_by_for_accounts(dids, &self.db).await
    }

    pub async fn set_account_invites_disabled(&self, did: &str, disabled: bool) -> Result<()> {
        invite::set_account_invites_disabled(did, disabled, &self.db).await
    }

    pub async fn disable_invite_codes(&self, opts: DisableInviteCodesOpts) -> Result<()> {
        invite::disable_invite_codes(opts, &self.db).await
    }

    // Passwords
    // ----------

    pub async fn create_app_password(
        &self,
        did: String,
        name: String,
    ) -> Result<CreateAppPasswordOutput> {
        password::create_app_password(did, name, &self.db).await
    }

    pub async fn list_app_passwords(&self, did: &str) -> Result<Vec<(String, String)>> {
        password::list_app_passwords(did, &self.db).await
    }

    pub async fn verify_account_password(&self, did: &str, password_str: &String) -> Result<bool> {
        password::verify_account_password(did, password_str, &self.db).await
    }

    pub async fn verify_app_password(
        &self,
        did: &str,
        password_str: &str,
    ) -> Result<Option<String>> {
        password::verify_app_password(did, password_str, &self.db).await
    }

    pub async fn reset_password(&self, opts: ResetPasswordOpts) -> Result<()> {
        let did = email_token::assert_valid_token_and_find_did(
            EmailTokenPurpose::ResetPassword,
            &opts.token,
            None,
            &self.db,
        )
        .await?;
        self.update_account_password(UpdateAccountPasswordOpts {
            did,
            password: opts.password,
        })
        .await
    }

    pub async fn update_account_password(&self, opts: UpdateAccountPasswordOpts) -> Result<()> {
        let UpdateAccountPasswordOpts { did, .. } = opts;
        let password_encrypted = password::gen_salt_and_hash(opts.password)?;
        try_join!(
            password::update_user_password(
                UpdateUserPasswordOpts {
                    did: did.clone(),
                    password_encrypted
                },
                &self.db
            ),
            email_token::delete_email_token(&did, EmailTokenPurpose::ResetPassword, &self.db),
            auth::revoke_refresh_tokens_by_did(&did, &self.db)
        )?;
        Ok(())
    }

    pub async fn revoke_app_password(&self, did: String, name: String) -> Result<()> {
        try_join!(
            password::delete_app_password(&did, &name, &self.db),
            auth::revoke_app_password_refresh_token(&did, &name, &self.db)
        )?;
        Ok(())
    }

    // Email Tokens
    // ----------
    pub async fn confirm_email(&self, opts: ConfirmEmailOpts<'_>) -> Result<()> {
        let ConfirmEmailOpts { did, token } = opts;
        email_token::assert_valid_token(
            did,
            EmailTokenPurpose::ConfirmEmail,
            token,
            None,
            &self.db,
        )
        .await?;
        let now = rsky_common::now();
        try_join!(
            email_token::delete_email_token(did, EmailTokenPurpose::ConfirmEmail, &self.db),
            account::set_email_confirmed_at(did, now, &self.db)
        )?;
        Ok(())
    }

    pub async fn update_email(&self, opts: UpdateEmailOpts) -> Result<()> {
        let UpdateEmailOpts { did, email } = opts;
        try_join!(
            account::update_email(&did, &email, &self.db),
            email_token::delete_all_email_tokens(&did, &self.db)
        )?;
        Ok(())
    }

    pub async fn assert_valid_email_token(
        &self,
        did: &str,
        purpose: EmailTokenPurpose,
        token: &str,
    ) -> Result<()> {
        email_token::assert_valid_token(did, purpose, token, None, &self.db).await
    }

    pub async fn assert_valid_email_token_and_cleanup(
        &self,
        did: &str,
        purpose: EmailTokenPurpose,
        token: &str,
    ) -> Result<()> {
        email_token::assert_valid_token(did, purpose, token, None, &self.db).await?;
        email_token::delete_email_token(did, purpose, &self.db).await
    }

    pub async fn create_email_token(
        &self,
        did: &str,
        purpose: EmailTokenPurpose,
    ) -> Result<String> {
        email_token::create_email_token(did, purpose, &self.db).await
    }
}

impl AccountStore for AccountManager {
    fn authenticate_account(
        &self,
        credentials: SignInCredentials,
        device_id: DeviceId,
    ) -> Pin<Box<dyn Future<Output = Result<Option<AccountInfo>, OAuthError>> + Send + '_>> {
        let identifier = credentials.username;
        let password = credentials.password;
        let remember = credentials.remember;
        Box::pin(async move {
            let (user, app_password, is_soft_deleted) =
                self.login(identifier, password).await.unwrap();
            if app_password.is_some() {
                return Err(OAuthError::InvalidRequestError(
                    "App passwords are not allowed".to_string(),
                ));
            }
            let did = Sub::new(user.did).unwrap();
            device_account::create_or_update(
                &self.db,
                device_id.clone(),
                did.clone(),
                remember.unwrap_or(false),
            )
            .await
            .unwrap();

            self.get_device_account(device_id, did).await
        })
    }

    fn add_authorized_client(
        &self,
        device_id: DeviceId,
        sub: Sub,
        client_id: OAuthClientId,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + '_>> {
        Box::pin(async move {
            match device_account::add_authorized_client(&self.db, device_id, sub, client_id).await {
                Ok(_) => Ok(()),
                Err(_) => Err(OAuthError::RuntimeError("".to_string())),
            }
        })
    }

    fn get_device_account(
        &self,
        device_id: DeviceId,
        sub: Sub,
    ) -> Pin<Box<dyn Future<Output = Result<Option<AccountInfo>, OAuthError>> + Send + '_>> {
        let audience = Audience::Single(env::var("PDS_SERVICE_DID").unwrap());
        Box::pin(async move {
            match device_account::get_account_info(device_id, sub, audience, &self.db).await {
                Ok(account_info) => Ok(account_info),
                Err(_) => Err(OAuthError::RuntimeError("".to_string())),
            }
        })
    }

    fn remove_device_account(
        &self,
        device_id: DeviceId,
        sub: Sub,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + '_>> {
        Box::pin(async move {
            match device_account::remove_qb(device_id, sub, &self.db).await {
                Ok(_) => Ok(()),
                Err(_) => Err(OAuthError::RuntimeError("".to_string())),
            }
        })
    }

    fn list_device_accounts(
        &self,
        device_id: DeviceId,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<AccountInfo>, OAuthError>> + Send + '_>> {
        let audience = Audience::Single(env::var("PDS_SERVICE_DID").unwrap());
        let device_id = device_id.clone();
        Box::pin(async move {
            match device_account::list_remembered_devices(&self.db, device_id, audience).await {
                Ok(account_infos) => Ok(account_infos),
                Err(_) => Err(OAuthError::RuntimeError("".to_string())),
            }
        })
    }
}

impl RequestStore for AccountManager {
    fn create_request(
        &mut self,
        id: RequestId,
        data: RequestData,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + '_>> {
        Box::pin(async move {
            match authorization_request::create_qb(id, data, &self.db).await {
                Ok(_) => Ok(()),
                Err(_) => return Err(OAuthError::RuntimeError("".to_string())),
            }
        })
    }

    fn read_request(
        &self,
        id: &RequestId,
    ) -> Pin<Box<dyn Future<Output = Result<Option<RequestData>, OAuthError>> + Send + '_>> {
        let id = id.clone();
        Box::pin(async move {
            match authorization_request::read_qb(id, &self.db).await {
                Ok(result) => match result {
                    None => Ok(None),
                    Some(result) => Ok(Some(authorization_request::row_to_request_data(result))),
                },
                Err(error) => Err(OAuthError::RuntimeError("".to_string())),
            }
        })
    }

    fn update_request(
        &mut self,
        id: RequestId,
        data: UpdateRequestData,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + '_>> {
        Box::pin(async move {
            match authorization_request::update_qb(id, data, &self.db).await {
                Ok(_) => Ok(()),
                Err(_) => Err(OAuthError::RuntimeError("".to_string())),
            }
        })
    }

    fn delete_request(
        &mut self,
        id: RequestId,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + '_>> {
        Box::pin(async move {
            match authorization_request::remove_by_id_qb(id, &self.db).await {
                Ok(_) => Ok(()),
                Err(_) => Err(OAuthError::RuntimeError("".to_string())),
            }
        })
    }

    fn find_request_by_code(
        &self,
        code: Code,
    ) -> Pin<Box<dyn Future<Output = Option<FoundRequestResult>> + Send + '_>> {
        Box::pin(async move {
            let result = authorization_request::find_by_code_qb(&self.db, code)
                .await
                .unwrap_or_else(|error| None);
            match result {
                None => None,
                Some(result) => {
                    Some(authorization_request::row_to_found_request_result(result).unwrap())
                }
            }
        })
    }
}

impl DeviceStore for AccountManager {
    fn create_device(
        &mut self,
        device_id: DeviceId,
        data: DeviceData,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<(), OAuthError>> + Send + '_>> {
        Box::pin(async move {
            match device::create_device(device_id, data, &self.db).await {
                Ok(_) => Ok(()),
                Err(_) => Err(OAuthError::RuntimeError(
                    "Failed to create device".to_string(),
                )),
            }
        })
    }

    fn read_device(
        &self,
        device_id: DeviceId,
    ) -> Pin<
        Box<dyn Future<Output = std::result::Result<Option<DeviceData>, OAuthError>> + Send + '_>,
    > {
        Box::pin(async move {
            match device::read_device(device_id, &self.db).await {
                Ok(data) => Ok(data),
                Err(_) => Err(OAuthError::RuntimeError("".to_string())),
            }
        })
    }

    fn update_device(
        &mut self,
        device_id: DeviceId,
        data: PartialDeviceData,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<(), OAuthError>> + Send + '_>> {
        Box::pin(async move {
            match device::update_device(device_id, data, &self.db).await {
                Ok(_) => Ok(()),
                Err(_) => Err(OAuthError::RuntimeError("".to_string())),
            }
        })
    }

    fn delete_device(
        &mut self,
        device_id: DeviceId,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<(), OAuthError>> + Send + '_>> {
        Box::pin(async move {
            match device::delete_device(device_id, &self.db).await {
                Ok(_) => Ok(()),
                Err(_) => Err(OAuthError::RuntimeError("".to_string())),
            }
        })
    }
}

impl TokenStore for AccountManager {
    fn create_token(
        &mut self,
        token_id: TokenId,
        data: TokenData,
        refresh_token: Option<RefreshToken>,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + '_>> {
        Box::pin(async move {
            match refresh_token {
                None => {
                    token::create_qb(&self.db, token_id, data, refresh_token)
                        .await
                        .unwrap();
                    Ok(())
                }
                Some(refresh_token) => {
                    let count = used_refresh_token::count_qb(refresh_token.clone(), &self.db)
                        .await
                        .unwrap();
                    if count > 0 {
                        return Err(OAuthError::RuntimeError(
                            "Refresh token already in use".to_string(),
                        ));
                    }

                    token::create_qb(&self.db, token_id, data, Some(refresh_token))
                        .await
                        .unwrap();
                    Ok(())
                }
            }
        })
    }

    fn read_token(
        &self,
        token_id: TokenId,
    ) -> Pin<Box<dyn Future<Output = Result<Option<TokenInfo>, OAuthError>> + Send + '_>> {
        let audience = Audience::Single(env::var("PDS_SERVICE_DID").unwrap());
        Box::pin(async move {
            let opts = FindByQbOpts {
                id: None,
                code: None,
                token_id: Some(token_id.val()),
                current_refresh_token: None,
            };
            let row = token::read_token(&self.db, opts, audience).await.unwrap();
            match row {
                None => Ok(None),
                Some(row) => Ok(Some(row)),
            }
        })
    }

    fn delete_token(
        &mut self,
        token_id: TokenId,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + '_>> {
        Box::pin(async move {
            // Will cascade to used_refresh_token (used_refresh_token_fk)
            match token::remove_qb(&self.db, token_id).await {
                Ok(_) => Ok(()),
                Err(_) => Err(OAuthError::RuntimeError(
                    "Failed to delete token".to_string(),
                )),
            }
        })
    }

    fn rotate_token(
        &mut self,
        token_id: TokenId,
        new_token_id: TokenId,
        new_refresh_token: RefreshToken,
        new_data: NewTokenData,
    ) -> Pin<Box<dyn Future<Output = Result<(), OAuthError>> + Send + '_>> {
        let token_id = token_id.val();
        Box::pin(async move {
            let (id, current_refresh_token) =
                token::for_rotate(&self.db, token_id.clone()).await.unwrap();

            used_refresh_token::insert_qb(current_refresh_token, id, &self.db)
                .await
                .unwrap();

            let count = used_refresh_token::count_qb(new_refresh_token.clone(), &self.db)
                .await
                .unwrap();

            if count > 0 {
                // Do NOT throw (we don't want the transaction to be rolled back)
            } else {
                token::rotate_qb(
                    &self.db,
                    token_id,
                    new_token_id,
                    new_refresh_token,
                    new_data,
                )
                .await
                .unwrap();
            }
            Ok(())
        })
    }

    fn find_token_by_refresh_token(
        &self,
        refresh_token: RefreshToken,
    ) -> Pin<Box<dyn Future<Output = Result<Option<TokenInfo>, OAuthError>> + Send + '_>> {
        let audience = Audience::Single(env::var("PDS_SERVICE_DID").unwrap());
        Box::pin(async move {
            let used = used_refresh_token::find_by_token_qb(refresh_token.clone(), &self.db)
                .await
                .unwrap();

            let search = match used {
                None => FindByQbOpts {
                    id: None,
                    code: None,
                    token_id: None,
                    current_refresh_token: Some(refresh_token.val()),
                },
                Some(used) => FindByQbOpts {
                    id: Some(used.token_id),
                    code: None,
                    token_id: None,
                    current_refresh_token: None,
                },
            };

            let row = token::read_token(&self.db, search, audience).await.unwrap();
            match row {
                None => Ok(None),
                Some(row) => Ok(Some(row)),
            }
        })
    }

    fn find_token_by_code(
        &self,
        code: Code,
    ) -> Pin<Box<dyn Future<Output = Result<Option<TokenInfo>, OAuthError>> + Send + '_>> {
        let audience = Audience::Single(env::var("PDS_SERVICE_DID").unwrap());
        Box::pin(async move {
            let opts = FindByQbOpts {
                id: None,
                code: Some(code.into_inner()),
                token_id: None,
                current_refresh_token: None,
            };
            match token::read_token(&self.db, opts, audience).await {
                Ok(token_info) => Ok(token_info),
                Err(error) => Err(OAuthError::RuntimeError("DB Exception".to_string())),
            }
        })
    }
}

impl ClientStore for AccountManager {
    fn find_client(&self, client_id: OAuthClientId) -> Result<OAuthClientMetadata, OAuthError> {
        unimplemented!()
    }
}

pub struct SharedAccountManager {
    pub account_manager: Arc<RwLock<AccountManagerCreator>>,
    pub service_did: String,
    pub jwt_key: String,
}
impl Clone for SharedAccountManager {
    fn clone(&self) -> Self {
        Self {
            account_manager: Arc::clone(&self.account_manager),
            service_did: self.service_did.clone(),
            jwt_key: self.jwt_key.clone(),
        }
    }
}
