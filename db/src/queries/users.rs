use diesel::{
    ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, QueryResult, RunQueryDsl,
};
use schema::schema::users;
use time::OffsetDateTime;

use crate::types::UserId;

pub fn create(conn: &mut PgConnection, now: OffsetDateTime) -> QueryResult<UserId> {
    diesel::insert_into(users::table)
        .values(users::created_at.eq(now))
        .returning(users::id)
        .get_result(conn)
}

pub fn find_by_id(conn: &mut PgConnection, id: i32) -> QueryResult<Option<UserId>> {
    users::table
        .filter(users::id.eq(id))
        .select(users::id)
        .get_result(conn)
        .optional()
}
