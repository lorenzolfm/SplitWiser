use diesel::{ExpressionMethods, PgConnection, QueryResult, RunQueryDsl};
use schema::schema::user_revenues;
use time::OffsetDateTime;

use crate::types::UserRevenueId;

pub struct CreateParams<'a> {
    pub user_id: i32,
    pub amount_cents: i64,
    pub description: Option<&'a str>,
    pub incoming_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

pub fn create(conn: &mut PgConnection, p: &CreateParams) -> QueryResult<UserRevenueId> {
    diesel::insert_into(user_revenues::table)
        .values((
            user_revenues::user_id.eq(p.user_id),
            user_revenues::amount_cents.eq(p.amount_cents),
            user_revenues::incoming_at.eq(p.incoming_at),
            user_revenues::description.eq(p.description),
            user_revenues::created_at.eq(p.created_at),
        ))
        .returning(user_revenues::id)
        .get_result(conn)
}
