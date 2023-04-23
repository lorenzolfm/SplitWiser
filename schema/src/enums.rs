#[derive(Clone, Copy, Debug, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::UserExpensesChargeMethod"]
pub enum UserExpensesChargeMethod {
    Even,
    Proportional,
    Full,
}
