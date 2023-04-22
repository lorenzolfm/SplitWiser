// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "user_expenses_charge_method"))]
    pub struct UserExpensesChargeMethod;
}

diesel::table! {
    user_expense_installments (id) {
        id -> Int4,
        user_expense_id -> Int4,
        charged_at -> Timestamptz,
        amount -> Int8,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::UserExpensesChargeMethod;

    user_expenses (id) {
        id -> Int4,
        created_by -> Int4,
        amount -> Int8,
        description -> Nullable<Text>,
        chargee_user_id -> Int4,
        charged_user_id -> Int4,
        charge_method -> UserExpensesChargeMethod,
        begin_charging_at -> Timestamptz,
    }
}

diesel::table! {
    user_payments (id) {
        id -> Int4,
        created_by -> Int4,
        amount -> Int8,
        payee_user_id -> Int4,
        payer_user_id -> Int4,
        payed_at -> Timestamptz,
    }
}

diesel::table! {
    user_revenues (id) {
        id -> Int4,
        user_id -> Int4,
        amount -> Int8,
        incoming_at -> Timestamptz,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        lightning_address -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(user_expense_installments -> user_expenses (user_expense_id));
diesel::joinable!(user_revenues -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    user_expense_installments,
    user_expenses,
    user_payments,
    user_revenues,
    users,
);
