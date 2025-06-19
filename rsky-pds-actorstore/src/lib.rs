#[macro_use]
extern crate serde;

pub mod actor_store;
pub mod read_after_write;
pub mod sequencer;

pub mod db {
    pub use rsky_pds_models::db::*;
}
pub mod models {
    pub use rsky_pds_models::models::*;
}
pub mod schema {
    pub use rsky_pds_models::schema::*;
}
pub mod auth_verifier {
    pub use rsky_pds_accountmanager::auth_verifier::*;
}
pub mod image {
    pub use rsky_pds_common::image::*;
}

pub mod account_manager {
    pub use rsky_pds_accountmanager::account_manager::*;
}
pub mod pipethrough {
    pub use rsky_pds_accountmanager::pipethrough::*;
}
pub mod xrpc_server {
    pub use rsky_pds_accountmanager::xrpc_server::*;
}
pub mod crawlers {
    pub use rsky_pds_common::crawlers::*;
}

use crate::read_after_write::viewer::{LocalViewer, LocalViewerCreator, LocalViewerCreatorParams};
use tokio::sync::RwLock;
pub struct SharedLocalViewer {
    pub local_viewer: RwLock<LocalViewerCreator>,
}

pub use rsky_pds_common::APP_USER_AGENT;

// Use lazy_static! because the size of EventEmitter is not known at compile time
use event_emitter_rs::EventEmitter;
use lazy_static::lazy_static;
lazy_static! {
    // Export the emitter with `pub` keyword
    pub static ref EVENT_EMITTER: RwLock<EventEmitter> = RwLock::new(EventEmitter::new());
}
