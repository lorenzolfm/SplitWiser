use db::{
    enums::UserExpensesChargeMethod,
    queries::{user_expense_installments, user_expenses},
    types::{UserExpenseId, UserId},
};
use time::{Duration, OffsetDateTime};

pub async fn create(db: &db::Db) -> Result<UserId, db::Error> {
    Ok(db
        .write::<_, db::Error, _>(move |conn| {
            db::queries::users::create(conn, OffsetDateTime::now_utc())
        })
        .await?)
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

#[derive(Debug, thiserror::Error)]
pub enum CreateExpenseError {
    #[error("Time conversion error: {0:?}")]
    TimeError(time::error::ComponentRange),
    #[error("Database error: {0:?}")]
    DbError(db::Error),
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
) -> Result<CreateExpenseOutcome, CreateExpenseError> {
    let begin_charging_at = OffsetDateTime::from_unix_timestamp(begin_charging_at)
        .map_err(CreateExpenseError::TimeError)?;

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
        .map_err(CreateExpenseError::DbError)?;

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
