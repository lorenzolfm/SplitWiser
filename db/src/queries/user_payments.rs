use diesel::{ExpressionMethods, PgConnection, QueryResult, RunQueryDsl};
use schema::schema::user_payments;
use time::OffsetDateTime;

use crate::types::{UserId, UserPaymentId};

pub struct CreateParams {
    created_by_id: UserId,
    amount: i64,
    payee_user_id: UserId,
    payer_user_id: UserId,
    payed_at: OffsetDateTime,
}

pub fn create(conn: &mut PgConnection, p: &CreateParams) -> QueryResult<UserPaymentId> {
    diesel::insert_into(user_payments::table)
        .values((
            user_payments::created_by.eq(*p.created_by_id),
            user_payments::amount.eq(p.amount),
            user_payments::payee_user_id.eq(*p.payee_user_id),
            user_payments::payer_user_id.eq(*p.payer_user_id),
            user_payments::payed_at.eq(p.payed_at),
        ))
        .returning(user_payments::id)
        .get_result(conn)
}
