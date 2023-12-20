#![allow(clippy::declare_interior_mutable_const)]
#![rustfmt::skip]

use core::time::Duration;

pub const DB_CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
pub const GLOBAL_EXPIRE_TIME: Duration = Duration::from_secs(1800);
pub const GLOBAL_INTERVAL: Duration = Duration::from_secs(
    #[cfg(debug_assertions)]
    60,
    #[cfg(not(debug_assertions))]
    600,
);

pub const PING_INTERVAL: Duration = Duration::from_millis(18320);
pub const PING_TIMEOUT: Duration = Duration::from_millis(28560);

pub const EMITTER_SECRET: &str = include_str!("../../emitter.secret");
