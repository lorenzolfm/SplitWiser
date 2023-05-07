use diesel::{ExpressionMethods, PgConnection, QueryResult, RunQueryDsl};
use schema::schema::user_payments;
use time::OffsetDateTime;

use crate::types::UserPaymentId;

pub struct CreateParams {
    pub created_by: i32,
    pub amount_cents: i64,
    pub payee_user_id: i32,
    pub payer_user_id: i32,
    pub payed_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

pub fn create(conn: &mut PgConnection, p: &CreateParams) -> QueryResult<UserPaymentId> {
    diesel::insert_into(user_payments::table)
        .values((
            user_payments::created_by.eq(p.created_by),
            user_payments::amount_cents.eq(p.amount_cents),
            user_payments::payee_user_id.eq(p.payee_user_id),
            user_payments::payer_user_id.eq(p.payer_user_id),
            user_payments::payed_at.eq(p.payed_at),
            user_payments::created_at.eq(p.created_at),
        ))
        .returning(user_payments::id)
        .get_result(conn)
}

pub fn delete(conn: &mut PgConnection, id: i32, user_id: i32) -> QueryResult<UserPaymentId> {
    diesel::delete(user_payments::table)
        .filter(user_payments::id.eq(id))
        .filter(user_payments::created_by.eq(user_id))
        .returning(user_payments::id)
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

        fn setup_with_amount(amount_cents: i64) -> QueryResult<super::UserPaymentId> {
            let mut conn = test::conn();
            let u0 = *users::create(&mut conn, OffsetDateTime::now_utc()).unwrap();
            let u1 = *users::create(&mut conn, OffsetDateTime::now_utc()).unwrap();

            super::create(
                &mut conn,
                &super::CreateParams {
                    created_by: u0,
                    amount_cents,
                    payee_user_id: u0,
                    payer_user_id: u1,
                    payed_at: OffsetDateTime::now_utc(),
                    created_at: OffsetDateTime::now_utc(),
                },
            )
        }

        #[test]
        fn amount_cents_should_be_greater_than_zero() {
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
        fn payee_and_payer_cannot_be_the_same() {
            let mut conn = test::conn();
            let u0 = *users::create(&mut conn, OffsetDateTime::now_utc()).unwrap();

            let res = super::create(
                &mut conn,
                &super::CreateParams {
                    created_by: u0,
                    amount_cents: 1,
                    payee_user_id: u0,
                    payer_user_id: u0,
                    payed_at: OffsetDateTime::now_utc(),
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
                    created_by: u0,
                    amount_cents: 1,
                    payee_user_id: u0,
                    payer_user_id: u1,
                    payed_at: OffsetDateTime::now_utc(),
                    created_at: OffsetDateTime::now_utc(),
                },
            );

            assert!(res.is_ok());
        }
    }
}
