use bigdecimal::ToPrimitive;
use db::Db;
use time::OffsetDateTime;

#[derive(Debug, thiserror::Error)]
pub enum GetError {
    #[error("Database error: {0:?}")]
    Database(#[from] db::Error),
    #[error("Time conversion error: {0:?}")]
    Time(#[from] time::error::ComponentRange),
    #[error("BigDecimal to i64 error")]
    BigDecimalToI64,
}

pub async fn get(
    db: &Db,
    payer: i32,
    payee: i32,
    from_timestamp: u64,
    to_timestamp: u64,
) -> Result<(), GetError> {
    let from = OffsetDateTime::from_unix_timestamp(from_timestamp as i64)?;
    let to = OffsetDateTime::from_unix_timestamp(to_timestamp as i64)?;

    db.read(move |conn| {
        let payer_expense =
            db::queries::user_expenses::find_for_period(conn, payee, payer, from, to)?;

        let Some(payer_revenue) =
            db::queries::user_revenues::find_for_period(conn, payer, from, to)?
                .unwrap_or_default()
                .to_i64() else {
                    return Err(GetError::BigDecimalToI64);
                };

        let Some(payee_revenue) =
            db::queries::user_revenues::find_for_period(conn, payee, from, to)?
                .unwrap_or_default()
                .to_i64() else {
                    return Err(GetError::BigDecimalToI64);
                };

        let total_revenue = payer_revenue + payee_revenue;

        let payer_share = payer_revenue / total_revenue;

        let (mut proportional_for_payer, mut even_for_payer, mut full_for_payer) = (0, 0, 0);

        for e in &payer_expense {
            match e.charge_method {
                db::enums::UserExpensesChargeMethod::Proportional => {
                    proportional_for_payer += e.amount_cents
                }
                db::enums::UserExpensesChargeMethod::Even => even_for_payer += e.amount_cents,
                db::enums::UserExpensesChargeMethod::Full => full_for_payer += e.amount_cents,
            }
        }

        let total = full_for_payer as f64
            + (even_for_payer as f64 / 2_f64)
            + (proportional_for_payer as f64 / payer_share as f64);

        Ok(total)
    })
    .await
    .unwrap();

    Ok(())
}
