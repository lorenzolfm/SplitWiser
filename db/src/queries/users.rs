use diesel::{ExpressionMethods, PgConnection, QueryResult, RunQueryDsl};
use schema::schema::users;
use time::OffsetDateTime;

pub fn create(conn: &mut PgConnection, now: OffsetDateTime) -> QueryResult<i32> {
    diesel::insert_into(users::table)
        .values(users::created_at.eq(now))
        .returning(users::id)
        .get_result(conn)
}
