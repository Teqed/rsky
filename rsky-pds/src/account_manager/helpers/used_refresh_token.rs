use crate::models::models::pds as models;
use crate::schema::pds::used_refresh_token::dsl as RefreshSchema;
use anyhow::Result;
use diesel::*;
use diesel::{insert_into, QueryDsl, RunQueryDsl, SelectableHelper};
use rsky_oauth::oauth_provider::token::refresh_token::RefreshToken;

pub async fn insert_qb(
    refresh_token: String,
    token_id: String,
    db: &deadpool_diesel::Pool<
        deadpool_diesel::Manager<SqliteConnection>,
        deadpool_diesel::sqlite::Object,
    >,
) -> Result<()> {
    db.get()
        .await
        .expect("Failed to get DB connection")
        .interact(move |conn| {
            let rows: Vec<models::UsedRefreshToken> = vec![models::UsedRefreshToken {
                token_id,
                refresh_token,
            }];
            insert_into(RefreshSchema::used_refresh_token)
                .values(&rows)
                .execute(conn)
        })
        .await
        .expect("Failed to insert used refresh token")?;
    Ok(())
}

pub async fn find_by_token_qb(
    refresh_token: RefreshToken,
    db: &deadpool_diesel::Pool<
        deadpool_diesel::Manager<SqliteConnection>,
        deadpool_diesel::sqlite::Object,
    >,
) -> Result<Option<models::UsedRefreshToken>> {
    let refresh_token = refresh_token.val();
    let result = db
        .get()
        .await
        .expect("Failed to get DB connection")
        .interact(move |conn| {
            RefreshSchema::used_refresh_token
                .filter(RefreshSchema::refreshToken.eq(refresh_token))
                .select(models::UsedRefreshToken::as_select())
                .first(conn)
                .optional()
        })
        .await
        .expect("Failed to find used refresh token")?;
    Ok(result)
}

pub async fn count_qb(
    refresh_token: RefreshToken,
    db: &deadpool_diesel::Pool<
        deadpool_diesel::Manager<SqliteConnection>,
        deadpool_diesel::sqlite::Object,
    >,
) -> Result<u64> {
    let refresh_token = refresh_token.val();
    let result = db
        .get()
        .await
        .expect("Failed to get DB connection")
        .interact(move |conn| {
            RefreshSchema::used_refresh_token
                .filter(RefreshSchema::refreshToken.eq(refresh_token))
                .select(models::UsedRefreshToken::as_select())
                .get_results(conn)
        })
        .await
        .expect("Failed to count used refresh tokens")?;
    Ok(result.len() as u64)
}
