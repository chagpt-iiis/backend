use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio_postgres::types::Json;

use crate::libs::db::{get_connection, BB8Error};

const GET_REPERTOIRE: &str = "select data from repertoire";
const UPDATE_REPERTOIRE: &str = "insert into repertoire (data) values ($1) on conflict ((1)) do update set data = excluded.data";

#[derive(Debug, Serialize, Deserialize)]
pub struct Program {
    pub id: u32,
    pub name: String,
    pub performer: String,
    pub time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repertoire {
    pub programs: Vec<Program>,
    pub current: u32,
}

pub static REPERTOIRE: RwLock<Option<Repertoire>> = RwLock::new(None);

pub async fn init() -> Result<(), BB8Error> {
    let mut conn = get_connection().await?;
    let stmt = conn.prepare_static(GET_REPERTOIRE.into()).await?;

    let row = conn.query_one(&stmt, &[]).await?;
    let Json(repertoire) = row.try_get(0)?;

    *REPERTOIRE.write() = Some(repertoire);
    Ok(())
}

pub async fn update(data: Repertoire) -> Result<String, BB8Error> {
    let mut conn = get_connection().await?;
    let stmt = conn.prepare_static(UPDATE_REPERTOIRE.into()).await?;

    let data = Json(data);
    conn.execute(&stmt, &[&data]).await?;

    let payload = format!(
        r#"4{{"type":"repertoire","programs":{},"current":{}}}"#,
        serde_json::to_string(&data.0.programs).unwrap_or_else(|_| "[]".into()),
        data.0.current
    );

    *REPERTOIRE.write() = Some(data.0);

    Ok(payload)
}
