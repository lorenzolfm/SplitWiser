macro_rules! diesel_ids {
    ($($id:ident),+) => {
        $(
            #[derive(Debug, Clone, Copy, diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)]
            #[diesel(sql_type = diesel::sql_types::Integer)]
            pub struct $id(i32);

            impl std::ops::Deref for $id {
                type Target = i32;
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl diesel::deserialize::FromSql<diesel::sql_types::Integer, diesel::pg::Pg> for $id {
                fn from_sql(bytes: diesel::backend::RawValue<diesel::pg::Pg>) -> diesel::deserialize::Result<Self> {
                    <i32 as diesel::deserialize::FromSql<diesel::sql_types::Integer, diesel::pg::Pg>>::from_sql(bytes).map(Self)
                }
            }
        )+
    };
}

diesel_ids!(
    UserId,
    UserRevenueId,
    UserPaymentId,
    UserExpenseId,
    UserExpenseInstallmentId
);
