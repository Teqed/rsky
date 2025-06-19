#[macro_use]
extern crate serde;

pub mod crawlers;
pub mod lexicon;
pub mod repo;
pub mod mailer;
pub mod image;
pub mod plc;

pub mod db {
    pub use rsky_pds_models::db::*;
}
pub mod models {
    pub use rsky_pds_models::models::*;
}
pub mod schema {
    pub use rsky_pds_models::schema::*;
}

use rsky_identity::IdResolver;
use tokio::sync::RwLock;

pub static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_HOMEPAGE"),
    "@",
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

pub struct SharedIdResolver {
    pub id_resolver: RwLock<IdResolver>,
}
