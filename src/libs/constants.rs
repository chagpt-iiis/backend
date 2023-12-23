#![allow(clippy::declare_interior_mutable_const)]
#![rustfmt::skip]

use core::time::Duration;

use actix_web::web::Bytes;

pub const BYTES_NULL: Bytes = Bytes::from_static(b"null");
pub const BYTES_TRUE: Bytes = Bytes::from_static(b"true");
pub const BYTES_FALSE: Bytes = Bytes::from_static(b"false");

pub const DB_CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);

pub const ETH_URL: &str = "https://www.blockchain.com/explorer/blocks/eth";
pub const ETH_TIMEOUT: Duration = Duration::from_secs(10);
pub const ETH_INTERNAL: Duration = Duration::from_secs(15);

pub const PING_INTERVAL: Duration = Duration::from_millis(18320);
pub const PING_TIMEOUT: Duration = Duration::from_millis(28560);

pub const ADMIN_SECRET: &str = include_str!("../../admin.secret");
pub const EMITTER_SECRET: &str = include_str!("../../emitter.secret");
pub const LOTTERY_SECRET: &str = include_str!("../../lottery.secret");
