use diesel::{r2d2::ConnectionManager, PgConnection};
use r2d2::Pool;

type ManagedConn = ConnectionManager<PgConnection>;

pub struct Db(pub(crate) Pool<ManagedConn>);
pub type Error = diesel::result::Error;

pub fn build(database_url: &str) -> Result<Db, r2d2::Error> {
    let pool = Pool::<ManagedConn>::builder()
        .max_size(2)
        .build(ConnectionManager::new(database_url))?;

    Ok(Db(pool))
}
