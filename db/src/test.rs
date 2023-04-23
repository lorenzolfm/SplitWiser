use diesel::{r2d2::ConnectionManager, Connection, PgConnection};
use once_cell::sync::Lazy;
use r2d2::Pool;

static URL: Lazy<String> =
    Lazy::new(|| std::env::var("DATABASE_URL").expect("DATABASE_URL not present"));

#[must_use]
pub fn conn() -> PgConnection {
    let mut conn = PgConnection::establish(&URL).expect("Could not establish database connection");
    conn.begin_test_transaction().unwrap();
    conn
}

#[must_use]
pub fn db() -> crate::db::Db {
    let connman = ConnectionManager::<PgConnection>::new(&URL as &str);
    let db = Pool::builder().max_size(1).build(connman).unwrap();
    db.get().unwrap().begin_test_transaction().unwrap();
    crate::db::Db(db)
}
