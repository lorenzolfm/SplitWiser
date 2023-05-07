use diesel::{ExpressionMethods, PgConnection, QueryResult, RunQueryDsl};
use schema::schema::user_expense_installments;
use time::OffsetDateTime;

use crate::types::UserExpenseId;

pub struct CreateParams {
    pub user_expense_id: UserExpenseId,
    pub charged_at: OffsetDateTime,
    pub amount_cents: i64,
}

pub fn create(conn: &mut PgConnection, installments: &[CreateParams]) -> QueryResult<usize> {
    let tuples = installments.iter().map(|p| {
        (
            user_expense_installments::user_expense_id.eq(*p.user_expense_id),
            user_expense_installments::charged_at.eq(p.charged_at),
            user_expense_installments::amount_cents.eq(p.amount_cents),
        )
    });

    diesel::insert_into(user_expense_installments::table)
        .values(tuples.collect::<Vec<_>>())
        .execute(conn)
}
