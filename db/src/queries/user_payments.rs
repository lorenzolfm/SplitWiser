use diesel::{ExpressionMethods, PgConnection, QueryResult, RunQueryDsl};
use schema::schema::user_payments;
use time::OffsetDateTime;

use crate::types::UserPaymentId;

pub struct CreateParams {
    pub created_by: i32,
    pub amount: i64,
    pub payee_user_id: i32,
    pub payer_user_id: i32,
    pub payed_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

pub fn create(conn: &mut PgConnection, p: &CreateParams) -> QueryResult<UserPaymentId> {
    diesel::insert_into(user_payments::table)
        .values((
            user_payments::created_by.eq(p.created_by),
            user_payments::amount.eq(p.amount),
            user_payments::payee_user_id.eq(p.payee_user_id),
            user_payments::payer_user_id.eq(p.payer_user_id),
            user_payments::payed_at.eq(p.payed_at),
            user_payments::created_at.eq(p.created_at),
        ))
        .returning(user_payments::id)
        .get_result(conn)
}
