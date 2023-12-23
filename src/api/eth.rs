use std::time::SystemTime;

use actix_web::web::{self, Bytes};
use bytes::Buf;
use bytestring::ByteString;
use serde::Deserialize;

use crate::libs::{
    chagpt::{admin::CURRENT_ADMIN, Emit},
    constants::{BYTES_NULL, LOTTERY_SECRET},
    eth,
};

#[derive(Deserialize)]
pub struct BlockRequest {
    secret: String,
}

pub async fn block(web::Json(req): web::Json<BlockRequest>) -> Bytes {
    let BlockRequest { secret } = req;

    if secret != LOTTERY_SECRET {
        return BYTES_NULL;
    }

    let Some(block) = eth::fetch() else {
        return BYTES_NULL;
    };

    let bt = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(block.time);
    let nt = SystemTime::now();
    let now = unsafe {
        nt.duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_unchecked()
            .as_millis()
    };

    let payload = Emit(ByteString::from(format!(
        r#"4{{"type":"lottery","block":{},"hash":"{}","blockTime":{},"now":{now}}}"#,
        block.height,
        block.hash,
        block.time * 1000,
    )));

    tracing::info!(target: "eth-request", "Block {} (blockTime: {bt:?}, now: {nt:?}) is taken", block.height);

    if let Some(ref addr) = *CURRENT_ADMIN.read() {
        if let Err(e) = addr.do_send(payload.clone()) {
            tracing::error!(target: "danmaku-to-admin", err = ?e);
        }
    }

    let mut payload = payload.0.into_bytes();
    payload.advance(1);
    payload
}
