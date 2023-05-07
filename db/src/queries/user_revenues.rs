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

pub fn delete(conn: &mut PgConnection, id: i32, user_id: i32) -> QueryResult<UserRevenueId> {
    diesel::delete(user_revenues::table)
        .filter(user_revenues::id.eq(id))
        .filter(user_revenues::user_id.eq(user_id))
        .returning(user_revenues::id)
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
}
