use db::{
    enums::UserExpensesChargeMethod,
    queries::{user_expense_installments, user_expenses, user_payments, user_revenues},
};
use time::{Duration, OffsetDateTime};
use tonic::Status;

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
    let id = db
        .write::<_, db::Error, _>(move |conn| {
            let total_amount = request.amount;
            let begin_charging_at =
                OffsetDateTime::from_unix_timestamp(request.begin_charging_at).unwrap();

            let charge_method = match request.method() {
                create_expense_request::Method::Even => UserExpensesChargeMethod::Even,
                create_expense_request::Method::Proportional => {
                    UserExpensesChargeMethod::Proportional
                }
                create_expense_request::Method::Full => UserExpensesChargeMethod::Full,
            };

            let user_expense_id = user_expenses::create(
                conn,
                &user_expenses::CreateParams {
                    created_by: request.created_by,
                    amount: total_amount as i64,
                    description: request.description.as_deref(),
                    chargee_user_id: request.chargee_user_id,
                    charged_user_id: request.charged_user_id,
                    begin_charging_at,
                    charge_method,
                },
            )?;

            let installments: Vec<_> = (0..request.installments)
                .map(|i| user_expense_installments::CreateParams {
                    user_expense_id,
                    charged_at: begin_charging_at + Duration::weeks(4 * (i as i64)),
                    amount: (total_amount / request.installments as u64) as i64,
                })
                .collect();

            user_expense_installments::create(conn, &installments)?;

            Ok(user_expense_id)
        })
        .await
        .map_err(|_| Status::internal("db error"))?;

    Ok(Id { id: *id })
}
