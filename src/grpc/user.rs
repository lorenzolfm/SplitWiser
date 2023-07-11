use db::enums::UserExpensesChargeMethod;
use tonic::Status;

use crate::features::user::{self, CreateExpenseOutcome, UserError};

use super::proto::{
    create_expense_request, CreateExpenseRequest, CreatePaymentRequest, CreateRevenueRequest, Id,
};

pub(super) async fn create(db: &db::Db) -> Result<Id, Status> {
    match user::create(db).await {
        Ok(id) => Ok(Id { id: *id }),
        Err(_) => Err(Status::internal("Database error")),
    }
}

pub(super) async fn create_revenue(
    db: &db::Db,
    request: CreateRevenueRequest,
) -> Result<Id, Status> {
    match user::create_revenue(
        db,
        user::CreateRevenueParams {
            user_id: request.user_id,
            amount_cents: request.amount_cents as i64,
            description: request.description,
            incoming_at: request.incoming_at,
        },
    )
    .await
    {
        Ok(id) => Ok(Id { id: *id }),
        Err(_e @ UserError::Time(_)) => Err(Status::out_of_range(
            "Invalid timestamp for begin_charging_at",
        )),
        Err(_e @ UserError::Database(_)) => Err(Status::internal("Database error")),
    }
}

pub(super) async fn create_payment(
    db: &db::Db,
    request: CreatePaymentRequest,
) -> Result<Id, Status> {
    match user::create_payment(
        db,
        user::CreatePaymentParams {
            created_by: request.created_by,
            amount_cents: request.amount_cents as i64,
            payee_user_id: request.payee_user_id,
            payer_user_id: request.payer_user_id,
            payed_at: request.payed_at,
        },
    )
    .await
    {
        Ok(id) => Ok(Id { id: *id }),
        Err(_e @ UserError::Time(_)) => Err(Status::out_of_range(
            "Invalid timestamp for begin_charging_at",
        )),
        Err(_e @ UserError::Database(_)) => Err(Status::internal("Database error")),
    }
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
        crate::features::user::CreateExpenseParams {
            amount_cents: request.amount_cents as i64,
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
        Err(_e @ UserError::Time(_)) => Err(Status::out_of_range(
            "Invalid timestamp for begin_charging_at",
        )),
        Err(_e @ UserError::Database(_)) => Err(Status::internal("Database error")),
    }
}
