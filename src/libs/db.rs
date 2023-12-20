use std::{future::Future, time::SystemTime};

use bb8_postgres::{bb8, PostgresConnectionManager};
use tokio_postgres::{
    types::{FromSql, Type},
    NoTls, Row,
};

use super::{
    constants::{DB_CONNECTION_TIMEOUT, GLOBAL_EXPIRE_TIME, GLOBAL_INTERVAL},
    response::StdError,
};

pub type ConnectionManager = PostgresConnectionManager<NoTls>;
pub type Pool = bb8::Pool<ConnectionManager>;
pub type PooledConnection = bb8::PooledConnection<'static, ConnectionManager>;
pub type DBError = tokio_postgres::Error;
pub type BB8Error = bb8::RunError<DBError>;
pub type DBResult<T> = Result<T, DBError>;

static mut POOL: Option<Pool> = None;

pub async fn init_db() {
    let mut config = tokio_postgres::Config::new();
    config
        .host_path("/tmp")
        .user("test")
        .dbname("postgres")
        .connect_timeout(DB_CONNECTION_TIMEOUT);

    let manager = PostgresConnectionManager::new(config, NoTls);

    let pool = Pool::builder()
        .connection_timeout(DB_CONNECTION_TIMEOUT)
        .build(manager)
        .await
        .unwrap();

    unsafe {
        assert!(POOL.is_none());
        POOL = Some(pool);
    }
}

#[inline(always)]
pub fn get_connection() -> impl Future<Output = Result<PooledConnection, BB8Error>> {
    unsafe { POOL.as_ref().unwrap_unchecked().get() }
}

#[inline(always)]
pub async fn insert_connection(
    conn: &mut Option<PooledConnection>,
) -> Result<&mut PooledConnection, BB8Error> {
    Ok(if let Some(db) = conn {
        db
    } else {
        conn.insert(get_connection().await?)
    })
}

#[inline]
pub fn transfer_type<'a, T, U>(row: &'a Row, idx: usize) -> DBResult<U>
where
    T: FromSql<'a> + TryInto<U>,
    <T as TryInto<U>>::Error: StdError + Send + Sync + 'static,
{
    row.try_get::<'a, usize, T>(idx)?
        .try_into()
        .map_err(|e| DBError::new(tokio_postgres::error::Kind::FromSql(idx), Some(Box::new(e))))
}

#[allow(clippy::cognitive_complexity)]
pub async fn expired_token_cleaner() {
    const SQL_CLEAN: &str = "delete from tokens where token_latest < $1";
    loop {
        tracing::debug!(target: "expired-token-cleaner", "start clean");

        let t = SystemTime::now() - GLOBAL_EXPIRE_TIME;
        match get_connection().await {
            Err(e) => tracing::warn!(target: "expired-token-cleaner", bb8_error = ?e),
            Ok(mut conn) => {
                let e: DBResult<()> = try {
                    let stmt = conn.prepare_static(SQL_CLEAN.into()).await?;
                    let n = conn.execute(&stmt, &[&t]).await?;
                    tracing::debug!(target: "expired-token-cleaner", "cleaned \x1b[32m{n}\x1b[0m tokens!");
                };
                if let Err(e) = e {
                    tracing::warn!(target: "expired-token-cleaner", psql_error = ?e);
                }
            }
        }

        tokio::time::sleep(GLOBAL_INTERVAL).await;
    }
}

#[repr(transparent)]
pub struct JsonChecked<'a>(pub &'a [u8]);

impl<'a> FromSql<'a> for JsonChecked<'a> {
    #[inline]
    fn from_sql(_: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn StdError + Sync + Send>> {
        if let [1, rest @ ..] = raw {
            Ok(Self(rest))
        } else {
            Err("database JSONB error".into())
        }
    }

    #[inline]
    fn accepts(_: &Type) -> bool {
        true
    }
}
