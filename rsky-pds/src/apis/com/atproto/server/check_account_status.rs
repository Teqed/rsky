use crate::account_manager::AccountManager;
use crate::actor_store::blob::fs::BlobStoreFs;
use crate::actor_store::ActorStore;
use crate::apis::com::atproto::server::is_valid_did_doc_for_service;
use crate::apis::ApiError;
use crate::auth_verifier::AccessFull;
use crate::db::DbConn;
use anyhow::Result;
use futures::try_join;
use rocket::serde::json::Json;
use rocket::State;
use rsky_lexicon::com::atproto::server::CheckAccountStatusOutput;
use std::path::PathBuf;

async fn inner_check_account_status(
    auth: AccessFull,
    blob_config: &State<PathBuf>,
    db: DbConn,
    account_manager: AccountManager,
) -> Result<CheckAccountStatusOutput> {
    let requester = auth.access.credentials.unwrap().did.unwrap();

    let mut actor_store = ActorStore::new(
        requester.clone(),
        BlobStoreFs::new(requester.clone(), blob_config.inner().clone()),
        db,
    );
    let repo_root = {
        let storage_guard = actor_store.storage.read().await;
        storage_guard.get_root_detailed().await?
    };
    let repo_blocks = {
        let storage_guard = actor_store.storage.read().await;
        storage_guard.count_blocks().await?
    };
    let (indexed_records, imported_blobs, expected_blobs) = try_join!(
        actor_store.record.record_count(),
        actor_store.blob.blob_count(),
        actor_store.blob.record_blob_count(),
    )?;

    let (activated, valid_did) = try_join!(
        account_manager.is_account_activated(&requester),
        is_valid_did_doc_for_service(requester.clone())
    )?;

    Ok(CheckAccountStatusOutput {
        activated,
        valid_did,
        repo_commit: repo_root.cid.to_string(),
        repo_rev: repo_root.rev,
        repo_blocks,
        indexed_records,
        private_state_values: 0,
        expected_blobs,
        imported_blobs,
    })
}

#[tracing::instrument(skip_all)]
#[rocket::get("/xrpc/com.atproto.server.checkAccountStatus")]
pub async fn check_account_status(
    auth: AccessFull,
    blob_config: &State<PathBuf>,
    db: DbConn,
    account_manager: AccountManager,
) -> Result<Json<CheckAccountStatusOutput>, ApiError> {
    match inner_check_account_status(auth, blob_config, db, account_manager).await {
        Ok(res) => Ok(Json(res)),
        Err(error) => {
            tracing::error!("Internal Error: {error}");
            Err(ApiError::RuntimeError)
        }
    }
}
