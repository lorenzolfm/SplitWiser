use bigdecimal::ToPrimitive;
use db::{
    enums::UserExpensesChargeMethod,
    queries::{user_expenses, user_incomes},
    Db,
};
use time::OffsetDateTime;

pub struct Statement {
    pub income: i64,
    pub pays: i64,
    pub paid: i64,
    pub owes: i64,
    pub share: String,
}

#[derive(Debug, thiserror::Error)]
pub enum StatementError {
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
) -> Result<Statement, StatementError> {
    let from = OffsetDateTime::from_unix_timestamp(from_timestamp as i64)?;
    let until = OffsetDateTime::from_unix_timestamp(to_timestamp as i64)?;

    db.read(move |conn| {
        let Some(payer_income) = user_incomes::find_for_period(conn, payer, from, until)?.unwrap_or_default().to_i64() else {
            return Err(StatementError::BigDecimalToI64);
        };
        let Some(payee_income) = user_incomes::find_for_period(conn, payee, from, until)?.unwrap_or_default().to_i64() else {
            return Err(StatementError::BigDecimalToI64);
        };

        let total_income = dbg!(payer_income + payee_income);

        let Some(paid) = user_expenses::find_for_period_and_charge_method(
            conn,
            payer,
            payee,
            from,
            until,
            UserExpensesChargeMethod::Proportional,
        )?.unwrap_or_default().to_i64() else {
            return Err(StatementError::BigDecimalToI64);
        };
        let Some(payee_expenses) = user_expenses::find_for_period_and_charge_method(
            conn,
            payee,
            payer,
            from,
            until,
            UserExpensesChargeMethod::Proportional,
        )?.unwrap_or_default().to_i64() else {
            return Err(StatementError::BigDecimalToI64);
        };

        let total_expenses = dbg!(paid + payee_expenses);
        let payer_share = dbg!(payer_income as f64 / total_income as f64);
        let pays = dbg!((total_expenses as f64 * payer_share) as i64);

        Ok(Statement {
             income: payer_income,
             pays,
             paid,
             owes: pays - paid,
             share: format!("{:.2}", (payer_share * 100.0)),
        })
    })
    .await
}

mod test {
    #[tokio::test]
    async fn test_get() {
        const ONE_BRL_IN_CENTS: f64 = 100_f64;

        let db = db::test::db();

        let now = time::macros::datetime!(2023-06-01 00:00).assume_utc();
        let incoming_at = time::macros::datetime!(2023-06-01 00:01).assume_utc();

        let user_1 = db::queries::users::create(&mut db.conn().await.unwrap(), now).unwrap();
        let user_2 = db::queries::users::create(&mut db.conn().await.unwrap(), now).unwrap();

        db::queries::user_incomes::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_incomes::CreateParams {
                user_id: *user_1,
                amount_cents: (6049.17 * ONE_BRL_IN_CENTS) as i64,
                description: None,
                incoming_at,
                created_at: now,
            },
        )
        .unwrap();

        db::queries::user_incomes::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_incomes::CreateParams {
                user_id: *user_1,
                amount_cents: (1100_f64 * ONE_BRL_IN_CENTS) as i64,
                description: None,
                incoming_at,
                created_at: now,
            },
        )
        .unwrap();

        db::queries::user_incomes::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_incomes::CreateParams {
                user_id: *user_1,
                amount_cents: (5182.86 * ONE_BRL_IN_CENTS) as i64,
                description: None,
                incoming_at,
                created_at: now,
            },
        )
        .unwrap();

        db::queries::user_expenses::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_expenses::CreateParams {
                amount_cents: (150.0 * ONE_BRL_IN_CENTS) as i64,
                begin_charging_at: now,
                charge_method: db::enums::UserExpensesChargeMethod::Proportional,
                chargee_user_id: *user_1,
                charged_user_id: *user_2,
                created_by: *user_1,
                created_at: now,
                description: Some("Bia"),
                installments: 1,
            },
        )
        .unwrap();

        db::queries::user_incomes::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_incomes::CreateParams {
                user_id: *user_2,
                amount_cents: (6049.17 * ONE_BRL_IN_CENTS) as i64,
                description: None,
                incoming_at,
                created_at: now,
            },
        )
        .unwrap();

        db::queries::user_incomes::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_incomes::CreateParams {
                user_id: *user_2,
                amount_cents: (5500_f64 * ONE_BRL_IN_CENTS) as i64,
                description: None,
                incoming_at,
                created_at: now,
            },
        )
        .unwrap();

        db::queries::user_incomes::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_incomes::CreateParams {
                user_id: *user_2,
                amount_cents: (1100_f64 * ONE_BRL_IN_CENTS) as i64,
                description: None,
                incoming_at,
                created_at: now,
            },
        )
        .unwrap();

        db::queries::user_incomes::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_incomes::CreateParams {
                user_id: *user_2,
                amount_cents: (200_f64 * ONE_BRL_IN_CENTS) as i64,
                description: None,
                incoming_at,
                created_at: now,
            },
        )
        .unwrap();

        db::queries::user_expenses::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_expenses::CreateParams {
                amount_cents: (40.9 * ONE_BRL_IN_CENTS) as i64,
                begin_charging_at: now,
                charge_method: db::enums::UserExpensesChargeMethod::Proportional,
                created_at: now,
                chargee_user_id: *user_2,
                charged_user_id: *user_1,
                created_by: *user_2,
                description: Some("Latam Pass"),
                installments: 1,
            },
        )
        .unwrap();

        db::queries::user_expenses::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_expenses::CreateParams {
                amount_cents: (39.9 * ONE_BRL_IN_CENTS) as i64,
                begin_charging_at: now,
                charge_method: db::enums::UserExpensesChargeMethod::Proportional,
                created_at: now,
                chargee_user_id: *user_2,
                charged_user_id: *user_1,
                created_by: *user_2,
                description: Some("Club smiles"),
                installments: 1,
            },
        )
        .unwrap();

        db::queries::user_expenses::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_expenses::CreateParams {
                amount_cents: (227.0 * ONE_BRL_IN_CENTS) as i64,
                begin_charging_at: now,
                charge_method: db::enums::UserExpensesChargeMethod::Proportional,
                created_at: now,
                chargee_user_id: *user_2,
                charged_user_id: *user_1,
                created_by: *user_2,
                description: Some("Mecanica"),
                installments: 1,
            },
        )
        .unwrap();

        db::queries::user_expenses::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_expenses::CreateParams {
                amount_cents: (267.27 * ONE_BRL_IN_CENTS) as i64,
                begin_charging_at: now,
                charge_method: db::enums::UserExpensesChargeMethod::Proportional,
                created_at: now,
                chargee_user_id: *user_2,
                charged_user_id: *user_1,
                created_by: *user_2,
                description: Some("Hippo"),
                installments: 1,
            },
        )
        .unwrap();

        db::queries::user_expenses::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_expenses::CreateParams {
                amount_cents: (90.0 * ONE_BRL_IN_CENTS) as i64,
                begin_charging_at: now,
                charge_method: db::enums::UserExpensesChargeMethod::Proportional,
                created_at: now,
                chargee_user_id: *user_2,
                charged_user_id: *user_1,
                created_by: *user_2,
                description: Some("Limpeza carro"),
                installments: 1,
            },
        )
        .unwrap();

        db::queries::user_expenses::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_expenses::CreateParams {
                amount_cents: (39.0 * ONE_BRL_IN_CENTS) as i64,
                begin_charging_at: now,
                charge_method: db::enums::UserExpensesChargeMethod::Proportional,
                created_at: now,
                chargee_user_id: *user_2,
                charged_user_id: *user_1,
                created_by: *user_2,
                description: Some("Imperatriz"),
                installments: 1,
            },
        )
        .unwrap();

        db::queries::user_expenses::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_expenses::CreateParams {
                amount_cents: (100.0 * ONE_BRL_IN_CENTS) as i64,
                begin_charging_at: now,
                charge_method: db::enums::UserExpensesChargeMethod::Proportional,
                created_at: now,
                chargee_user_id: *user_2,
                charged_user_id: *user_1,
                created_by: *user_2,
                description: Some("Gasosa"),
                installments: 1,
            },
        )
        .unwrap();

        db::queries::user_expenses::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_expenses::CreateParams {
                amount_cents: (150.36 * ONE_BRL_IN_CENTS) as i64,
                begin_charging_at: now,
                charge_method: db::enums::UserExpensesChargeMethod::Proportional,
                created_at: now,
                chargee_user_id: *user_2,
                charged_user_id: *user_1,
                created_by: *user_2,
                description: Some("Chas"),
                installments: 1,
            },
        )
        .unwrap();

        db::queries::user_expenses::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_expenses::CreateParams {
                amount_cents: (289.69 * ONE_BRL_IN_CENTS) as i64,
                begin_charging_at: now,
                charge_method: db::enums::UserExpensesChargeMethod::Proportional,
                created_at: now,
                chargee_user_id: *user_2,
                charged_user_id: *user_1,
                created_by: *user_2,
                description: Some("Gasosa"),
                installments: 1,
            },
        )
        .unwrap();

        db::queries::user_expenses::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_expenses::CreateParams {
                amount_cents: (55.9 * ONE_BRL_IN_CENTS) as i64,
                begin_charging_at: now,
                charge_method: db::enums::UserExpensesChargeMethod::Proportional,
                created_at: now,
                chargee_user_id: *user_2,
                charged_user_id: *user_1,
                created_by: *user_2,
                description: Some("Netflix"),
                installments: 1,
            },
        )
        .unwrap();

        db::queries::user_expenses::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_expenses::CreateParams {
                amount_cents: (34.9 * ONE_BRL_IN_CENTS) as i64,
                begin_charging_at: now,
                charge_method: db::enums::UserExpensesChargeMethod::Proportional,
                created_at: now,
                chargee_user_id: *user_2,
                charged_user_id: *user_1,
                created_by: *user_2,
                description: Some("Apple tv"),
                installments: 1,
            },
        )
        .unwrap();

        db::queries::user_expenses::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_expenses::CreateParams {
                amount_cents: (21.9 * ONE_BRL_IN_CENTS) as i64,
                begin_charging_at: now,
                charge_method: db::enums::UserExpensesChargeMethod::Proportional,
                created_at: now,
                chargee_user_id: *user_2,
                charged_user_id: *user_1,
                created_by: *user_2,
                description: Some("Prime video canais"),
                installments: 1,
            },
        )
        .unwrap();

        db::queries::user_expenses::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_expenses::CreateParams {
                amount_cents: (14.9 * ONE_BRL_IN_CENTS) as i64,
                begin_charging_at: now,
                charge_method: db::enums::UserExpensesChargeMethod::Proportional,
                created_at: now,
                chargee_user_id: *user_2,
                charged_user_id: *user_1,
                created_by: *user_2,
                description: Some("Amazon prime"),
                installments: 1,
            },
        )
        .unwrap();

        db::queries::user_expenses::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_expenses::CreateParams {
                amount_cents: (299.33 * ONE_BRL_IN_CENTS) as i64,
                begin_charging_at: now,
                charge_method: db::enums::UserExpensesChargeMethod::Proportional,
                created_at: now,
                chargee_user_id: *user_2,
                charged_user_id: *user_1,
                created_by: *user_2,
                description: Some("Marmitas fit mercado sao jorge"),
                installments: 1,
            },
        )
        .unwrap();

        let res = super::get(
            &db,
            *user_1,
            *user_2,
            now.unix_timestamp() as u64,
            time::OffsetDateTime::now_utc().unix_timestamp() as u64,
        )
        .await
        .unwrap();

        assert_eq!(res.income, (12332.02 * ONE_BRL_IN_CENTS) as i64);
        assert_eq!(res.paid, (150.0 * ONE_BRL_IN_CENTS) as i64);
        assert_eq!(res.pays, (891.82 * ONE_BRL_IN_CENTS) as i64);
        assert_eq!(res.share, String::from("48.97"));
        assert_eq!(res.owes, (741.82 * ONE_BRL_IN_CENTS) as i64);
    }
}
