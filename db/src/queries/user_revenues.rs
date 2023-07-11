use bigdecimal::BigDecimal;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, QueryResult, RunQueryDsl};
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

pub fn delete(conn: &mut PgConnection, id: i32, user_id: i32) -> QueryResult<UserRevenueId> {
    diesel::delete(user_revenues::table)
        .filter(user_revenues::id.eq(id))
        .filter(user_revenues::user_id.eq(user_id))
        .returning(user_revenues::id)
        .get_result(conn)
}

pub fn find_for_period(
    conn: &mut PgConnection,
    user_id: i32,
    from: OffsetDateTime,
    until: OffsetDateTime,
) -> QueryResult<Option<BigDecimal>> {
    user_revenues::table
        .filter(user_revenues::user_id.eq(user_id))
        .filter(user_revenues::incoming_at.gt(from))
        .filter(user_revenues::incoming_at.le(until))
        .select(diesel::dsl::sum(user_revenues::amount_cents))
        .get_result(conn)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{queries::users, test};

    mod create {
        use super::*;
        use diesel::QueryResult;

        fn test_with_amount(amount_cents: i64) -> QueryResult<super::UserRevenueId> {
            let mut conn = test::conn();
            let user_id = *users::create(&mut conn, OffsetDateTime::now_utc()).unwrap();

            super::create(
                &mut conn,
                &super::CreateParams {
                    user_id,
                    amount_cents,
                    description: None,
                    incoming_at: OffsetDateTime::now_utc(),
                    created_at: OffsetDateTime::now_utc(),
                },
            )
        }

        #[test]
        fn amount_cents_should_be_greater_than_zero() {
            let res = test_with_amount(-1);
            assert!(matches!(
                res.err(),
                Some(diesel::result::Error::DatabaseError(_, _))
            ));

            let res = test_with_amount(0);
            assert!(matches!(
                res.err(),
                Some(diesel::result::Error::DatabaseError(_, _))
            ));

            let res = test_with_amount(1);
            assert!(res.is_ok());
        }
    }

    mod revenue_for_period {
        use bigdecimal::BigDecimal;
        use time::OffsetDateTime;

        #[test]
        fn test_sum() {
            let mut conn = crate::test::conn();
            let user_id =
                *crate::queries::users::create(&mut conn, OffsetDateTime::now_utc()).unwrap();
            let from = time::macros::datetime!(2023-06-01 00:00).assume_utc();
            let incoming_at = time::macros::datetime!(2023-06-01 00:01).assume_utc();

            super::create(
                &mut conn,
                &super::CreateParams {
                    user_id,
                    amount_cents: 69,
                    description: None,
                    incoming_at,
                    created_at: OffsetDateTime::now_utc(),
                },
            )
            .unwrap();

            super::create(
                &mut conn,
                &super::CreateParams {
                    user_id,
                    amount_cents: 420,
                    description: None,
                    incoming_at,
                    created_at: OffsetDateTime::now_utc(),
                },
            )
            .unwrap();

            let expected = Some(BigDecimal::from(489));
            let actual =
                super::find_for_period(&mut conn, user_id, from, OffsetDateTime::now_utc())
                    .unwrap();

            assert_eq!(actual, expected);
        }
    }
}
