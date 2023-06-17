use diesel::{ExpressionMethods, PgConnection, QueryDsl, QueryResult, Queryable, RunQueryDsl};
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

#[derive(Queryable)]
pub struct Installment {
    pub amount_cents: i64,
    pub charged_at: OffsetDateTime,
}

pub fn find_by_user_expense_id(conn: &mut PgConnection, id: i32) -> QueryResult<Vec<Installment>> {
    user_expense_installments::table
        .filter(user_expense_installments::user_expense_id.eq(id))
        .select((
            user_expense_installments::amount_cents,
            user_expense_installments::charged_at,
        ))
        .get_results(conn)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test;

    mod create {
        use super::*;
        use crate::queries::users;
        use diesel::QueryResult;

        fn setup_with_amount(amount_cents: i64) -> QueryResult<usize> {
            let mut conn = test::conn();
            let u0 = users::create(&mut conn, OffsetDateTime::now_utc()).unwrap();
            let u1 = users::create(&mut conn, OffsetDateTime::now_utc()).unwrap();

            let user_expense_id = crate::queries::user_expenses::create(
                &mut conn,
                &crate::queries::user_expenses::CreateParams {
                    amount_cents: 1000,
                    created_by: *u0,
                    description: None,
                    chargee_user_id: *u0,
                    charged_user_id: *u1,
                    begin_charging_at: OffsetDateTime::now_utc(),
                    charge_method: crate::enums::UserExpensesChargeMethod::Even,
                    created_at: OffsetDateTime::now_utc(),
                    installments: 1,
                },
            )
            .unwrap();

            super::create(
                &mut conn,
                &[super::CreateParams {
                    amount_cents,
                    user_expense_id,
                    charged_at: OffsetDateTime::now_utc(),
                }],
            )
        }

        #[test]
        fn amount_cents_should_be_greate_than_zero() {
            let res = setup_with_amount(-1);
            assert!(matches!(
                res.err(),
                Some(diesel::result::Error::DatabaseError(_, _))
            ));

            let res = setup_with_amount(0);
            assert!(matches!(
                res.err(),
                Some(diesel::result::Error::DatabaseError(_, _))
            ));

            let res = setup_with_amount(1);
            assert!(res.is_ok());
        }
    }
}
