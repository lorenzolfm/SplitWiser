use diesel::{ExpressionMethods, PgConnection, QueryResult, RunQueryDsl};
use schema::{enums::UserExpensesChargeMethod, schema::user_expenses};
use time::OffsetDateTime;

use crate::types::UserExpenseId;

pub struct CreateParams<'a> {
    pub created_by: i32,
    pub amount_cents: i64,
    pub description: Option<&'a str>,
    pub chargee_user_id: i32,
    pub charged_user_id: i32,
    pub begin_charging_at: OffsetDateTime,
    pub charge_method: UserExpensesChargeMethod,
    pub created_at: OffsetDateTime,
}

pub fn create(conn: &mut PgConnection, p: &CreateParams) -> QueryResult<UserExpenseId> {
    diesel::insert_into(user_expenses::table)
        .values((
            user_expenses::created_by.eq(p.created_by),
            user_expenses::amount_cents.eq(p.amount_cents),
            user_expenses::description.eq(p.description),
            user_expenses::chargee_user_id.eq(p.chargee_user_id),
            user_expenses::charged_user_id.eq(p.charged_user_id),
            user_expenses::begin_charging_at.eq(p.begin_charging_at),
            user_expenses::charge_method.eq(p.charge_method),
            user_expenses::created_at.eq(p.created_at),
        ))
        .returning(user_expenses::id)
        .get_result(conn)
}

pub fn delete(conn: &mut PgConnection, id: i32, user_id: i32) -> QueryResult<UserExpenseId> {
    diesel::delete(user_expenses::table)
        .filter(user_expenses::id.eq(id))
        .filter(user_expenses::created_by.eq(user_id))
        .returning(user_expenses::id)
        .get_result(conn)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test;

    mod create {
        use super::*;
        use crate::queries::users;
        use diesel::QueryResult;

        fn setup_with_amount(amount_cents: i64) -> QueryResult<super::UserExpenseId> {
            let mut conn = test::conn();
            let u0 = *users::create(&mut conn, OffsetDateTime::now_utc()).unwrap();
            let u1 = *users::create(&mut conn, OffsetDateTime::now_utc()).unwrap();

            super::create(
                &mut conn,
                &super::CreateParams {
                    amount_cents,
                    created_by: u0,
                    description: None,
                    chargee_user_id: u0,
                    charged_user_id: u1,
                    begin_charging_at: OffsetDateTime::now_utc(),
                    charge_method: super::UserExpensesChargeMethod::Even,
                    created_at: OffsetDateTime::now_utc(),
                },
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

        #[test]
        fn chargee_and_charged_cannot_be_the_same() {
            let mut conn = test::conn();
            let u0 = *users::create(&mut conn, OffsetDateTime::now_utc()).unwrap();

            let res = super::create(
                &mut conn,
                &super::CreateParams {
                    amount_cents: 1,
                    created_by: u0,
                    description: None,
                    chargee_user_id: u0,
                    charged_user_id: u0,
                    begin_charging_at: OffsetDateTime::now_utc(),
                    charge_method: super::UserExpensesChargeMethod::Even,
                    created_at: OffsetDateTime::now_utc(),
                },
            );

            assert!(matches!(
                res.err(),
                Some(diesel::result::Error::DatabaseError(_, _))
            ));
        }

        #[test]
        fn happy_path() {
            let mut conn = test::conn();
            let u0 = *users::create(&mut conn, OffsetDateTime::now_utc()).unwrap();
            let u1 = *users::create(&mut conn, OffsetDateTime::now_utc()).unwrap();

            let res = super::create(
                &mut conn,
                &super::CreateParams {
                    amount_cents: 1,
                    created_by: u0,
                    description: None,
                    chargee_user_id: u0,
                    charged_user_id: u1,
                    begin_charging_at: OffsetDateTime::now_utc(),
                    charge_method: super::UserExpensesChargeMethod::Even,
                    created_at: OffsetDateTime::now_utc(),
                },
            );

            assert!(res.is_ok());
        }
    }
}
