CREATE TABLE user_incomes (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id),

    amount_cents BIGINT NOT NULL,
    incoming_at TIMESTAMPTZ NOT NULL,
    description TEXT,

    created_at TIMESTAMPTZ NOT NULL,

    CONSTRAINT user_revenue_amount_cents_is_greater_than_zero CHECK (amount_cents > 0)
);

CREATE TABLE user_payments (
    id SERIAL PRIMARY KEY,
    created_by INT NOT NULL REFERENCES users(id),

    amount_cents BIGINT NOT NULL,

    payee_user_id INT NOT NULL REFERENCES users(id),
    payer_user_id INT NOT NULL REFERENCES users(id),
    payed_at TIMESTAMPTZ NOT NULL,

    created_at TIMESTAMPTZ NOT NULL,

    CONSTRAINT user_payment_amount_cents_is_greater_than_zero CHECK (amount_cents > 0),
    CONSTRAINT payee_is_not_payer CHECK (payee_user_id <> payer_user_id)
);

CREATE TYPE user_expenses_charge_method as ENUM (
    'even',
    'proportional',
    'full'
);

CREATE TABLE user_expenses (
    id SERIAL PRIMARY KEY,
    created_by INT NOT NULL REFERENCES users(id),

    amount_cents BIGINT NOT NULL,
    description TEXT,

    chargee_user_id INT NOT NULL REFERENCES users(id),
    charged_user_id INT NOT NULL REFERENCES users(id),
    charge_method user_expenses_charge_method NOT NULL,
    begin_charging_at TIMESTAMPTZ NOT NULL,

    created_at TIMESTAMPTZ NOT NULL

    CONSTRAINT user_expense_amount_cents_is_greater_than_zero CHECK (amount_cents > 0),
    CONSTRAINT chargee_is_not_charged CHECK (charged_user_id <> chargee_user_id)
); 

CREATE TABLE user_expense_installments (
    id SERIAL PRIMARY KEY,
    user_expense_id INT NOT NULL REFERENCES user_expenses(id) ON DELETE CASCADE,
    charged_at TIMESTAMPTZ NOT NULL,

    amount_cents BIGINT NOT NULL

    CONSTRAINT user_expense_installment_amount_cents_is_greater_than_zero CHECK (amount_cents > 0)
);
