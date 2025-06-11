#[macro_use]
extern crate serde_derive;
extern crate core;
extern crate mailchecker;
extern crate serde;
use crate::read_after_write::viewer::{LocalViewer, LocalViewerCreator, LocalViewerCreatorParams};
use crate::sequencer::Sequencer;
use atrium_xrpc_client::reqwest::ReqwestClient;
use axum::{
    body::Body,
    extract::State,
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use event_emitter_rs::EventEmitter;
use http::StatusCode;
use lazy_static::lazy_static;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
pub mod account_manager;
pub mod actor_store;
pub mod apis;
pub mod auth;
pub mod config;
pub mod context;
pub mod crawlers;
pub mod db;
pub mod error;
pub mod handle;
pub mod image;
pub mod lexicon;
pub mod mailer;
pub mod models;
pub mod oauth;
pub mod pipethrough;
pub mod plc;
pub mod read_after_write;
pub mod repo;
pub mod schema;
pub mod sequencer;
pub mod well_known;
pub mod xrpc_server;

use crate::account_manager::{AccountManager, SharedAccountManager};
use crate::config::env_to_cfg;
use crate::crawlers::Crawlers;
use crate::db::DbPool;
use crate::models::{ErrorCode, ErrorMessageResponse, ServerVersion};
use diesel::prelude::*;

pub static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_HOMEPAGE"),
    "@",
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

pub struct SharedSequencer {
    pub sequencer: Arc<RwLock<Sequencer>>,
}

impl Clone for SharedSequencer {
    fn clone(&self) -> Self {
        Self {
            sequencer: self.sequencer.clone(),
        }
    }
}

pub struct SharedIdResolver {
    pub id_resolver: Arc<RwLock<IdResolver>>,
}

impl Clone for SharedIdResolver {
    fn clone(&self) -> Self {
        Self {
            id_resolver: self.id_resolver.clone(),
        }
    }
}

pub struct SharedLocalViewer {
    pub local_viewer: Arc<RwLock<LocalViewerCreator>>,
}

impl Clone for SharedLocalViewer {
    fn clone(&self) -> Self {
        Self {
            local_viewer: self.local_viewer.clone(),
        }
    }
}

pub struct SharedATPAgent {
    pub app_view_agent: Option<Arc<RwLock<AtpServiceClient<ReqwestClient>>>>,
}

impl Clone for SharedATPAgent {
    fn clone(&self) -> Self {
        Self {
            app_view_agent: self.app_view_agent.clone(),
        }
    }
}

// Use lazy_static! because the size of EventEmitter is not known at compile time
lazy_static! {
    // Export the emitter with `pub` keyword
    pub static ref EVENT_EMITTER: Arc<RwLock<EventEmitter>> = Arc::new(RwLock::new(EventEmitter::new()));
}

// TODO: Fix API error imports when converting APIs
// Temporarily use our own error enum that matches the one in the original code
#[derive(Debug)]
pub enum ApiError {
    XrpcError(StatusCode, String),
    AuthRequired,
    InvalidRequest,
    Forbidden,
    NotFound,
    PayloadTooLarge,
    RateLimitExceeded,
    InvalidToken,
    ExpiredToken,
    HandleNotAvailable,
    InvalidPassword,
    InvalidInviteCode,
    InviteCodeRequired,
    HandleRequired,
    InvalidHandle,
    UnsupportedDomain,
    AccountTakedown,
    RepositoryError,
    DiskSpaceExceeded,
    TempNotAvailable,
    AccountNotAvailable,
    EmailMismatch,
    LabelValueTooLong,
    RuntimeError,
}

impl ToString for ApiError {
    fn to_string(&self) -> String {
        match self {
            ApiError::RuntimeError => "Runtime Error".to_string(),
            ApiError::InvalidRequest => "Invalid Request".to_string(),
            ApiError::XrpcError(_, msg) => msg.clone(),
            _ => format!("{:?}", self),
        }
    }
}
// use crate::oauth::provider::{build_oauth_provider, AuthProviderOptions};
// use crate::oauth::{SharedOAuthProvider, SharedReplayStore};
use atrium_api::client::AtpServiceClient;
use atrium_xrpc_client::reqwest::ReqwestClientBuilder;
use diesel::sql_types::Int4;
use dotenvy::dotenv;
use http::header;
use rand::random;
use rsky_common::env::env_list;
use rsky_identity::types::{DidCache, IdentityResolverOpts};
use rsky_identity::IdResolver;
use rsky_oauth::oauth_provider::dpop::dpop_nonce::DpopNonceInput;
use rsky_oauth::oauth_provider::replay::replay_store_memory::ReplayStoreMemory;
use rsky_oauth::oauth_types::OAuthIssuerIdentifier;
use std::env;
use tokio::sync::RwLock;

/// Index route handler
async fn index() -> Html<&'static str> {
    Html(
        r#"
    .------..------..------..------.
    |R.--. ||S.--. ||K.--. ||Y.--. |
    | :(): || :/\: || :/\: || (\/) |
    | ()() || :\/: || :\/: || :\/: |
    | '--'R|| '--'S|| '--'K|| '--'Y|
    `------'`------'`------'`------'
    .------..------..------.
    |P.--. ||D.--. ||S.--. |
    | :/\: || :/\: || :/\: |
    | (__) || (__) || :\/: |
    | '--'P|| '--'D|| '--'S|
    `------'`------'`------'

    This is an atproto [https://atproto.com] Personal Data Server (PDS) running the rsky-pds codebase [https://github.com/blacksky-algorithms/rsky]

    Most API routes are under /xrpc/
    "#,
    )
}

/// Robots.txt route handler
async fn robots() -> &'static str {
    "# Hello!\n\n# Crawling the public API is allowed\nUser-agent: *\nAllow: /"
}

#[tracing::instrument(skip_all)]
async fn health(
    State(app_state): State<AppState>,
) -> Result<Json<ServerVersion>, (StatusCode, Json<ErrorMessageResponse>)> {
    let db_pool = &app_state.db_pool;
    let conn = match db_pool.get().await {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!("Failed to get DB connection: {err}");
            let internal_error = ErrorMessageResponse {
                code: Some(ErrorCode::ServiceUnavailable),
                message: Some(err.to_string()),
            };
            return Err((StatusCode::SERVICE_UNAVAILABLE, Json(internal_error)));
        }
    };

    let result = conn
        .interact(move |conn| {
            diesel::select(diesel::dsl::sql::<Int4>("1")) // SELECT 1;
                .load::<i32>(conn)
                .map(|v| v.into_iter().next().expect("no results"))
        })
        .await;

    match result {
        Ok(_) => {
            let env_version = env::var("VERSION").unwrap_or("0.3.0-beta.3".into());
            let version = ServerVersion {
                version: env_version,
            };
            Ok(Json(version))
        }
        Err(error) => {
            tracing::error!("Internal Error: {error}");
            let internal_error = ErrorMessageResponse {
                code: Some(ErrorCode::ServiceUnavailable),
                message: Some(error.to_string()),
            };
            Err((StatusCode::SERVICE_UNAVAILABLE, Json(internal_error)))
        }
    }
}

/// Custom error handler for API errors
pub async fn handle_api_error(err: ApiError) -> impl IntoResponse {
    let status = match &err {
        ApiError::XrpcError(status, _) => *status,
        ApiError::AuthRequired => StatusCode::UNAUTHORIZED,
        ApiError::InvalidRequest => StatusCode::BAD_REQUEST,
        ApiError::Forbidden => StatusCode::FORBIDDEN,
        ApiError::NotFound => StatusCode::NOT_FOUND,
        ApiError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        ApiError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
        ApiError::InvalidToken => StatusCode::UNAUTHORIZED,
        ApiError::ExpiredToken => StatusCode::UNAUTHORIZED,
        ApiError::HandleNotAvailable => StatusCode::BAD_REQUEST,
        ApiError::InvalidPassword => StatusCode::BAD_REQUEST,
        ApiError::InvalidInviteCode => StatusCode::BAD_REQUEST,
        ApiError::InviteCodeRequired => StatusCode::BAD_REQUEST,
        ApiError::HandleRequired => StatusCode::BAD_REQUEST,
        ApiError::InvalidHandle => StatusCode::BAD_REQUEST,
        ApiError::UnsupportedDomain => StatusCode::BAD_REQUEST,
        ApiError::AccountTakedown => StatusCode::FORBIDDEN,
        ApiError::RepositoryError => StatusCode::INTERNAL_SERVER_ERROR,
        ApiError::DiskSpaceExceeded => StatusCode::INSUFFICIENT_STORAGE,
        ApiError::TempNotAvailable => StatusCode::SERVICE_UNAVAILABLE,
        ApiError::AccountNotAvailable => StatusCode::BAD_REQUEST,
        ApiError::EmailMismatch => StatusCode::BAD_REQUEST,
        ApiError::LabelValueTooLong => StatusCode::BAD_REQUEST,
        ApiError::RuntimeError => StatusCode::INTERNAL_SERVER_ERROR,
    };

    let body = match &err {
        ApiError::XrpcError(_, body) => body.clone(),
        _ => err.to_string(),
    };

    // Create a proper Axum response with appropriate status code and body
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("Internal Server Error")))
}

pub struct AppConfig {
    pub db_url: String,
}

#[derive(Clone)]
pub struct AppState {
    pub sequencer: SharedSequencer,
    pub aws_sdk_config: aws_config::SdkConfig,
    pub id_resolver: SharedIdResolver,
    pub cfg: crate::config::ServerConfig,
    pub local_viewer: SharedLocalViewer,
    pub app_view_agent: SharedATPAgent,
    // TODO: Update the OAuth provider type when converting OAuth functionality
    // pub oauth_provider: SharedOAuthProvider,
    pub account_manager: SharedAccountManager,
    // pub replay_store: SharedReplayStore,
    pub db_pool: DbPool,
}

pub async fn build_app(cfg_option: Option<AppConfig>) -> (Router, SocketAddr) {
    dotenv().ok();

    let db_url = if let Some(cfg) = cfg_option {
        cfg.db_url
    } else {
        env::var("DATABASE_URL").unwrap_or("".into())
    };

    let cfg = env_to_cfg();

    // Create DB pool
    let db_pool = crate::db::establish_pool(&db_url).expect("Failed to establish database pool");

    let sequencer = SharedSequencer {
        sequencer: Arc::new(RwLock::new(Sequencer::new(
            Crawlers::new(cfg.service.hostname.clone(), cfg.crawlers.clone()),
            None,
        ))),
    };
    let sequencer_clone = sequencer.sequencer.clone();
    let mut background_sequencer = sequencer_clone.write().await.clone();
    tokio::spawn(async move { background_sequencer.start().await });

    let aws_sdk_config = aws_config::from_env()
        .endpoint_url(env::var("AWS_ENDPOINT").unwrap_or("localhost".to_owned()))
        .load()
        .await;

    let id_resolver = SharedIdResolver {
        id_resolver: Arc::new(RwLock::new(IdResolver::new(IdentityResolverOpts {
            timeout: None,
            plc_url: Some(
                env::var("PDS_DID_PLC_URL").unwrap_or("https://plc.directory".to_owned()),
            ),
            did_cache: Some(DidCache::new(None, None)),
            backup_nameservers: Some(env_list("PDS_HANDLE_BACKUP_NAMESERVERS")),
        }))),
    };

    // Keeping unused for other config purposes for now.
    let app_view_agent = match &cfg.bsky_app_view {
        None => SharedATPAgent {
            app_view_agent: None,
        },
        Some(bsky_app_view) => {
            let client = ReqwestClientBuilder::new(bsky_app_view.url.clone())
                .client(
                    reqwest::ClientBuilder::new()
                        .user_agent(APP_USER_AGENT)
                        .timeout(std::time::Duration::from_millis(1000))
                        .build()
                        .unwrap(),
                )
                .build();
            SharedATPAgent {
                app_view_agent: Some(Arc::new(RwLock::new(AtpServiceClient::new(client)))),
            }
        }
    };

    let local_viewer = SharedLocalViewer {
        local_viewer: Arc::new(RwLock::new(LocalViewer::creator(
            LocalViewerCreatorParams {
                pds_hostname: cfg.service.hostname.clone(),
                appview_agent: match &cfg.bsky_app_view {
                    None => None,
                    Some(bsky_app_view) => Some(bsky_app_view.url.clone()),
                },
                appview_did: match &cfg.bsky_app_view {
                    None => None,
                    Some(bsky_app_view) => Some(bsky_app_view.did.clone()),
                },
                appview_cdn_url_pattern: match &cfg.bsky_app_view {
                    None => None,
                    Some(bsky_app_view) => bsky_app_view.cdn_url_pattern.clone(),
                },
            },
        ))),
    };

    let dpop_secret = random::<[u8; 32]>();
    //Setup OAuth Provider
    // let oauth_provider = build_oauth_provider(AuthProviderOptions {
    //     issuer: OAuthIssuerIdentifier::new(
    //         env::var("OAUTH_ISSUER_IDENTIFIER").unwrap_or("https://pds.ripperoni.com".to_owned()),
    //     )
    //     .unwrap(),
    //     dpop_secret: Some(DpopNonceInput::Uint8Array(Vec::from(dpop_secret))),
    //     customization: None,
    //     redis: None,
    // })
    // .await;

    //Setup Account Manager
    let account_manager = SharedAccountManager {
        account_manager: AccountManager::creator(),
        service_did: "".to_string(),
        jwt_key: "".to_string(),
    };

    // let replay_store = SharedReplayStore {
    //     replay_store: Arc::new(RwLock::new(ReplayStoreMemory::new())),
    // };

    let app_state = AppState {
        sequencer,
        aws_sdk_config,
        id_resolver,
        cfg: cfg.clone(),
        local_viewer,
        app_view_agent,
        // oauth_provider,
        account_manager,
        // replay_store,
        db_pool: db_pool.clone(),
    };

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
        .allow_credentials(true);

    // Build main router
    let app = create_all_routes()
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Determine address to listen on
    let port = cfg.service.port;
    let addr = SocketAddr::from(([0, 0, 0, 0], port as u16));

    (app, addr)
}

fn create_xrpc_router() -> Router<AppState> {
    Router::new().route("/_health", get(health))
    // TODO
}

/// Creates all the routes for the XRPC API
pub fn create_all_routes() -> Router<AppState> {
    Router::new()
        // Base routes
        .route("/", get(index))
        .route("/robots.txt", get(robots))
        .route("/xrpc/_health", get(health))
        .nest("/xrpc", create_xrpc_router())
    // TODO: Implement static file serving with axum_static
    // .nest(
    //     "/@atproto/oauth-provider/~assets",
    //     axum_static::static_router(
    //         env::var("OAUTH_ASSET_LOCATION").unwrap_or("/pds/assets".to_owned()),
    //     ),
    // )
}

/// Function to run the server
pub async fn run_server(app: Router, addr: SocketAddr) -> anyhow::Result<()> {
    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))
}
