use db::{
    enums::UserExpensesChargeMethod,
    queries::{user_expense_installments, user_expenses},
    types::UserExpenseId,
};
use time::{Duration, OffsetDateTime};

pub struct CreateParams {
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
    CreateParams {
        total_amount,
        begin_charging_at,
        created_by,
        charged_user_id,
        chargee_user_id,
        charge_method,
        description,
        installments,
    }: CreateParams,
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

mod tests {
    // Test the error!
}
