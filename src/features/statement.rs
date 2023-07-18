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
) -> Result<i64, GetError> {
    let from = OffsetDateTime::from_unix_timestamp(from_timestamp as i64)?;
    let to = OffsetDateTime::from_unix_timestamp(to_timestamp as i64)?;

    let total = db
        .read(move |conn| {
            let payer_expense =
                db::queries::user_expenses::find_for_period(conn, payer, payee, from, to)?;
            let payee_expense =
                db::queries::user_expenses::find_for_period(conn, payee, payer, from, to)?;
            let total_expenses: i64 = payer_expense.iter().fold(0, |acc, e| acc + e.amount_cents)
                + payee_expense.iter().fold(0, |acc, e| acc + e.amount_cents);

            dbg!(total_expenses);

            let Some(payer_revenue) =
            db::queries::user_revenues::find_for_period(conn, payer, from, to)?.unwrap_or_default()
                .to_i64() else {
                    return Err(GetError::BigDecimalToI64);
                };

            let Some(payee_revenue) =
            db::queries::user_revenues::find_for_period(conn, payee, from, to)?
                .unwrap_or_default()
                .to_i64() else {
                    return Err(GetError::BigDecimalToI64);
                };

            let total_revenue = dbg!(payer_revenue + payee_revenue);
            let payer_share = dbg!(payer_revenue as f64 / total_revenue as f64);
            let pays = total_expenses as f64 * payer_share;

            dbg!(pays as i64);

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

            dbg!(payer_revenue, payer_share, proportional_for_payer);

            let total = full_for_payer as f64
                + (even_for_payer as f64 / 2_f64)
                + (proportional_for_payer as f64);

            Ok(pays as i64 - total as i64)
        })
        .await?;

    Ok(total)
}

mod test {
    macro_rules! assert_delta {
        ($x:expr, $y:expr, $d:expr) => {
            if !($x - $y < $d || $y - $x < $d) {
                panic!();
            }
        };
    }

    #[tokio::test]
    async fn test_get() {
        const ONE_BRL_IN_CENTS: f64 = 100_f64;

        let db = db::test::db();

        let now = time::macros::datetime!(2023-06-01 00:00).assume_utc();
        let incoming_at = time::macros::datetime!(2023-06-01 00:01).assume_utc();

        let user_1 = db::queries::users::create(&mut db.conn().await.unwrap(), now).unwrap();
        let user_2 = db::queries::users::create(&mut db.conn().await.unwrap(), now).unwrap();

        db::queries::user_revenues::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_revenues::CreateParams {
                user_id: *user_1,
                amount_cents: (6049.17 * ONE_BRL_IN_CENTS) as i64,
                description: None,
                incoming_at,
                created_at: now,
            },
        )
        .unwrap();

        db::queries::user_revenues::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_revenues::CreateParams {
                user_id: *user_1,
                amount_cents: (1100_f64 * ONE_BRL_IN_CENTS) as i64,
                description: None,
                incoming_at,
                created_at: now,
            },
        )
        .unwrap();

        db::queries::user_revenues::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_revenues::CreateParams {
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

        db::queries::user_revenues::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_revenues::CreateParams {
                user_id: *user_2,
                amount_cents: (6049.17 * ONE_BRL_IN_CENTS) as i64,
                description: None,
                incoming_at,
                created_at: now,
            },
        )
        .unwrap();

        db::queries::user_revenues::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_revenues::CreateParams {
                user_id: *user_2,
                amount_cents: (5500_f64 * ONE_BRL_IN_CENTS) as i64,
                description: None,
                incoming_at,
                created_at: now,
            },
        )
        .unwrap();

        db::queries::user_revenues::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_revenues::CreateParams {
                user_id: *user_2,
                amount_cents: (1100_f64 * ONE_BRL_IN_CENTS) as i64,
                description: None,
                incoming_at,
                created_at: now,
            },
        )
        .unwrap();

        db::queries::user_revenues::create(
            &mut db.conn().await.unwrap(),
            &db::queries::user_revenues::CreateParams {
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

        assert_eq!(res, (741.82 * ONE_BRL_IN_CENTS) as i64);
    }
}
