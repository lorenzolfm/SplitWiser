use db::{
    enums::UserExpensesChargeMethod,
    queries::{user_payments, user_revenues},
};
use time::OffsetDateTime;
use tonic::Status;

use crate::features::user::{CreateExpenseError, CreateExpenseOutcome};

use super::proto::{
    create_expense_request, CreateExpenseRequest, CreatePaymentRequest, CreateRevenueRequest, Id,
};

pub(super) async fn create(db: &db::Db) -> Result<Id, Status> {
    let id = db
        .write::<_, db::Error, _>(move |conn| {
            db::queries::users::create(conn, OffsetDateTime::now_utc())
        })
        .await
        .map_err(|_| Status::internal("db error"))?;

    Ok(Id { id: *id })
}

pub(super) async fn create_revenue(
    db: &db::Db,
    request: CreateRevenueRequest,
) -> Result<Id, Status> {
    let id = db
        .write(move |conn| {
            let incoming_at = OffsetDateTime::from_unix_timestamp(request.incoming_at).unwrap();

            user_revenues::create(
                conn,
                &user_revenues::CreateParams {
                    user_id: request.user_id,
                    amount: request.amount as i64,
                    description: &request.description,
                    incoming_at,
                    created_at: OffsetDateTime::now_utc(),
                },
            )
        })
        .await
        .map_err(|_| Status::internal("db error"))?;

    Ok(Id { id: *id })
}

pub(super) async fn create_payment(
    db: &db::Db,
    request: CreatePaymentRequest,
) -> Result<Id, Status> {
    let id = db
        .write(move |conn| {
            let payed_at = OffsetDateTime::from_unix_timestamp(request.payed_at).unwrap();

            user_payments::create(
                conn,
                &user_payments::CreateParams {
                    created_by: request.created_by,
                    amount: request.amount as i64,
                    payee_user_id: request.payee_user_id,
                    payer_user_id: request.payer_user_id,
                    payed_at,
                    created_at: OffsetDateTime::now_utc(),
                },
            )
        })
        .await
        .map_err(|_| Status::internal("db error"))?;

    Ok(Id { id: *id })
}

pub(super) async fn create_expense(
    db: &db::Db,
    request: CreateExpenseRequest,
) -> Result<Id, Status> {
    let charge_method = match request.method() {
        create_expense_request::Method::Even => UserExpensesChargeMethod::Even,
        create_expense_request::Method::Proportional => UserExpensesChargeMethod::Proportional,
        create_expense_request::Method::Full => UserExpensesChargeMethod::Full,
    };

    match crate::features::user::create_expense(
        db,
        crate::features::user::CreateParams {
            total_amount: request.amount as i64,
            begin_charging_at: request.begin_charging_at,
            created_by: request.created_by,
            charged_user_id: request.charged_user_id,
            chargee_user_id: request.chargee_user_id,
            charge_method,
            description: request.description,
            installments: request.installments,
        },
    )
    .await
    {
        Ok(CreateExpenseOutcome::Created(id)) => Ok(Id { id: *id }),
        Err(_e @ CreateExpenseError::TimeError(_)) => {
            Err(Status::internal("Invalid timestamp for begin_charging_at"))
        }
        Err(_e @ CreateExpenseError::DbError(_)) => Err(Status::internal("Database error")),
    }
}
