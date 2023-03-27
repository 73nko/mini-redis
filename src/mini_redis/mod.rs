mod commands;
mod request;

pub use commands::handle_connection;
pub use request::Request;
use tokio::sync::Mutex;

use std::{collections::HashMap, sync::Arc, time::Instant};

pub type KeyValueStore = Arc<Mutex<HashMap<String, (String, Option<Instant>)>>>;

#[cfg(test)]
mod tests;
