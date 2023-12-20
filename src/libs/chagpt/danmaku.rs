use std::time::SystemTime;

use bytestring::ByteString;
use serde::Deserialize;

use crate::libs::db::get_connection;

const INSERT_DANMAKU: &str =
    "insert into danmakus (content, time, color) values ($1, $2, $3) returning id";

pub struct Danmaku {
    pub id: u32,
    pub content: String,
    pub time: SystemTime,
    pub color: u32,
}

impl Danmaku {
    pub async fn insert(content: String, color: u32) -> Option<Self> {
        let mut conn = get_connection().await.ok()?;

        let stmt = conn.prepare_static(INSERT_DANMAKU.into()).await.ok()?;
        let time = SystemTime::now();
        let row = conn
            .query_one(&stmt, &[&content, &time, &(color as i32)])
            .await
            .ok()?;
        let id = row.try_get::<_, i32>(0).ok()? as u32;

        Some(Self {
            id,
            content,
            time,
            color,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "propose")]
    Propose { content: String, color: u32 },
}

#[derive(Clone)]
#[repr(transparent)]
pub struct BroadcastDanmaku(pub ByteString);

impl actix::Message for BroadcastDanmaku {
    type Result = ();
}
