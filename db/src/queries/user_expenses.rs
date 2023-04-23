use diesel::{ExpressionMethods, PgConnection, QueryResult, RunQueryDsl};
use schema::{enums::UserExpensesChargeMethod, schema::user_expenses};
use time::OffsetDateTime;

use crate::types::UserExpenseId;

pub struct CreateParams<'a> {
    pub created_by: i32,
    pub amount: i64,
    pub description: Option<&'a str>,
    pub chargee_user_id: i32,
    pub charged_user_id: i32,
    pub begin_charging_at: OffsetDateTime,
    pub charge_method: UserExpensesChargeMethod,
}

pub fn create(conn: &mut PgConnection, p: &CreateParams) -> QueryResult<UserExpenseId> {
    diesel::insert_into(user_expenses::table)
        .values((
            user_expenses::created_by.eq(p.created_by),
            user_expenses::amount.eq(p.amount),
            user_expenses::description.eq(p.description),
            user_expenses::chargee_user_id.eq(p.chargee_user_id),
            user_expenses::charged_user_id.eq(p.charged_user_id),
            user_expenses::begin_charging_at.eq(p.begin_charging_at),
            user_expenses::charge_method.eq(p.charge_method),
        ))
        .returning(user_expenses::id)
        .get_result(conn)
}