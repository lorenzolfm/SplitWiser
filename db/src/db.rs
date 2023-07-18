use diesel::{r2d2::ConnectionManager, PgConnection, QueryResult};
use r2d2::{Pool, PooledConnection};

type ManagedConn = ConnectionManager<PgConnection>;
type PooledConn = PooledConnection<ManagedConn>;

#[derive(Clone)]
pub struct Db(pub(crate) Pool<ManagedConn>);
pub type Error = diesel::result::Error;

pub fn build(database_url: &str) -> Result<Db, r2d2::Error> {
    let pool = Pool::<ManagedConn>::builder()
        .max_size(2)
        .build(ConnectionManager::new(database_url))?;

    Ok(Db(pool))
}

impl Db {
    #[cfg(feature = "test")]
    pub async fn conn(&self) -> Result<PooledConn, r2d2::Error> {
        let pool = self.0.clone();
        let res = tokio::task::spawn_blocking(move || pool.get());
        res.await.unwrap()
    }

    pub async fn read<R, E, F>(&self, f: F) -> Result<R, E>
    where
        R: 'static + Send,
        E: 'static + Send + From<diesel::result::Error>,
        F: 'static + Send + FnOnce(&mut PgConnection) -> Result<R, E>,
    {
        let db = self.clone();
        tokio::task::spawn_blocking(move || read(&mut *db.conn_or_rollback()?, f))
            .await
            .unwrap()
    }

    pub async fn write<R, E, F>(&self, f: F) -> Result<R, E>
    where
        R: 'static + Send,
        E: 'static + Send + From<diesel::result::Error>,
        F: 'static + Send + FnOnce(&mut PgConnection) -> Result<R, E>,
    {
        let db = self.clone();
        tokio::task::spawn_blocking(move || write(&mut *db.conn_or_rollback()?, f))
            .await
            .unwrap()
    }

    fn conn_or_rollback(&self) -> QueryResult<PooledConn> {
        self.0
            .get()
            .map_err(|_| diesel::result::Error::RollbackTransaction)
    }
}

#[cfg(not(feature = "test"))]
fn read<R, E, F>(conn: &mut PgConnection, f: F) -> Result<R, E>
where
    R: 'static + Send,
    E: 'static + Send + From<diesel::result::Error>,
    F: 'static + Send + FnOnce(&mut PgConnection) -> Result<R, E>,
{
    let builder = conn.build_transaction();
    let mut transaction = builder.serializable().read_only().deferrable();
    transaction.run(f)
}

#[cfg(not(feature = "test"))]
fn write<R, E, F>(conn: &mut PgConnection, f: F) -> Result<R, E>
where
    R: 'static + Send,
    E: 'static + Send + From<diesel::result::Error>,
    F: 'static + Send + FnOnce(&mut PgConnection) -> Result<R, E>,
{
    let builder = conn.build_transaction();
    let mut transaction = builder.repeatable_read().read_write();
    transaction.run(f)
}

#[cfg(feature = "test")]
fn write<R, E, F>(conn: &mut PgConnection, f: F) -> Result<R, E>
where
    R: 'static + Send,
    E: 'static + Send + From<diesel::result::Error>,
    F: 'static + Send + FnOnce(&mut PgConnection) -> Result<R, E>,
{
    diesel::Connection::transaction(conn, f)
}

#[allow(unused)]
#[cfg(feature = "test")]
fn read<R, E, F>(conn: &mut PgConnection, f: F) -> Result<R, E>
where
    R: 'static + Send,
    E: 'static + Send + From<diesel::result::Error>,
    F: 'static + Send + FnOnce(&mut PgConnection) -> Result<R, E>,
{
    diesel::Connection::transaction(conn, f)
}
