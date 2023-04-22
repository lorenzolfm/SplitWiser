use diesel::{ExpressionMethods, PgConnection, QueryResult, RunQueryDsl};
use schema::schema::user_expense_installments;
use time::OffsetDateTime;

use crate::types::{UserExpenseId, UserExpenseInstallmentId};

pub struct CreateParams {
    user_expense_id: UserExpenseId,
    charged_at: OffsetDateTime,
    amount: i64,
}

pub fn create(conn: &mut PgConnection, p: &CreateParams) -> QueryResult<UserExpenseInstallmentId> {
    diesel::insert_into(user_expense_installments::table)
        .values((
            user_expense_installments::user_expense_id.eq(*p.user_expense_id),
            user_expense_installments::charged_at.eq(p.charged_at),
            user_expense_installments::amount.eq(p.amount),
        ))
        .returning(user_expense_installments::id)
        .get_result(conn)
}
