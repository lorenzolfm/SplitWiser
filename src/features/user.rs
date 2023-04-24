use db::{
    enums::UserExpensesChargeMethod,
    queries::{user_expense_installments, user_expenses, user_payments, user_revenues},
    types::{UserExpenseId, UserId, UserPaymentId, UserRevenueId},
};
use time::{Duration, OffsetDateTime};

#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("Time conversion error: {0:?}")]
    TimeError(time::error::ComponentRange),
    #[error("Database error: {0:?}")]
    DbError(db::Error),
}

pub async fn create(db: &db::Db) -> Result<UserId, db::Error> {
    Ok(db
        .write::<_, db::Error, _>(move |conn| {
            db::queries::users::create(conn, OffsetDateTime::now_utc())
        })
        .await?)
}

pub struct CreateRevenueParams {
    pub user_id: i32,
    pub amount: i64,
    pub description: Option<String>,
    pub incoming_at: i64,
}

pub async fn create_revenue(
    db: &db::Db,
    CreateRevenueParams {
        user_id,
        amount,
        description,
        incoming_at,
    }: CreateRevenueParams,
) -> Result<UserRevenueId, UserError> {
    let created_at = OffsetDateTime::now_utc();
    let incoming_at =
        OffsetDateTime::from_unix_timestamp(incoming_at).map_err(UserError::TimeError)?;

    let id = db
        .write(move |conn| {
            user_revenues::create(
                conn,
                &user_revenues::CreateParams {
                    user_id,
                    amount,
                    description: description.as_deref(),
                    incoming_at,
                    created_at,
                },
            )
        })
        .await
        .map_err(UserError::DbError)?;

    Ok(id)
}

pub struct CreatePaymentParams {
    pub created_by: i32,
    pub amount: i64,
    pub payee_user_id: i32,
    pub payer_user_id: i32,
    pub payed_at: i64,
}

pub async fn create_payment(
    db: &db::Db,
    CreatePaymentParams {
        created_by,
        amount,
        payee_user_id,
        payer_user_id,
        payed_at,
    }: CreatePaymentParams,
) -> Result<UserPaymentId, UserError> {
    let created_at = OffsetDateTime::now_utc();
    let payed_at = OffsetDateTime::from_unix_timestamp(payed_at).map_err(UserError::TimeError)?;

    let id = db
        .write(move |conn| {
            user_payments::create(
                conn,
                &user_payments::CreateParams {
                    created_by,
                    amount,
                    payee_user_id,
                    payer_user_id,
                    payed_at,
                    created_at,
                },
            )
        })
        .await
        .map_err(UserError::DbError)?;

    Ok(id)
}

pub struct CreateExpenseParams {
    pub total_amount: i64,
    pub begin_charging_at: i64,
    pub created_by: i32,
    pub charged_user_id: i32,
    pub chargee_user_id: i32,
    pub charge_method: UserExpensesChargeMethod,
    pub description: Option<String>,
    pub installments: u32,
}

pub enum CreateExpenseOutcome {
    Created(UserExpenseId),
}

pub async fn create_expense(
    db: &db::Db,
    CreateExpenseParams {
        total_amount,
        begin_charging_at,
        created_by,
        charged_user_id,
        chargee_user_id,
        charge_method,
        description,
        installments,
    }: CreateExpenseParams,
) -> Result<CreateExpenseOutcome, UserError> {
    let created_at = OffsetDateTime::now_utc();
    let begin_charging_at =
        OffsetDateTime::from_unix_timestamp(begin_charging_at).map_err(UserError::TimeError)?;

    let id = db
        .write::<_, db::Error, _>(move |conn| {
            let user_expense_id = user_expenses::create(
                conn,
                &user_expenses::CreateParams {
                    created_by,
                    total_amount,
                    description: description.as_deref(),
                    chargee_user_id,
                    charged_user_id,
                    begin_charging_at,
                    charge_method,
                    created_at,
                },
            )?;

            let installments: Vec<_> = (0..installments)
                .map(|i| user_expense_installments::CreateParams {
                    user_expense_id,
                    charged_at: begin_charging_at + Duration::weeks(4 * (i as i64)),
                    amount: (total_amount / installments as i64),
                })
                .collect();

            user_expense_installments::create(conn, &installments)?;

            Ok(user_expense_id)
        })
        .await
        .map_err(UserError::DbError)?;

    Ok(CreateExpenseOutcome::Created(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    mod create_expense {
        use db::enums::UserExpensesChargeMethod;

        #[tokio::test]
        async fn test() {
            let db = db::test::db();

            let u1 = super::create(&db).await.unwrap();
            let u2 = super::create(&db).await.unwrap();

            super::create_expense(
                &db,
                super::CreateExpenseParams {
                    total_amount: 100,
                    begin_charging_at: 0,
                    created_by: *u1,
                    charged_user_id: *u1,
                    chargee_user_id: *u2,
                    charge_method: UserExpensesChargeMethod::Even,
                    description: Some("asdf".to_string()),
                    installments: 1,
                },
            )
            .await
            .unwrap();

            assert_eq!(2, 2);
        }
    }
}
