CREATE TABLE user_revenues (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id),

    amount BIGINT NOT NULL,
    incoming_at TIMESTAMPTZ NOT NULL,
    description TEXT,

    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE user_payments (
    id SERIAL PRIMARY KEY,
    created_by INT NOT NULL REFERENCES users(id),

    amount BIGINT NOT NULL,

    payee_user_id INT NOT NULL REFERENCES users(id),
    payer_user_id INT NOT NULL REFERENCES users(id),
    payed_at TIMESTAMPTZ NOT NULL,

    created_at TIMESTAMPTZ NOT NULL
);

CREATE TYPE user_expenses_charge_method as ENUM (
    'even',
    'proportional',
    'full'
);

CREATE TABLE user_expenses (
    id SERIAL PRIMARY KEY,
    created_by INT NOT NULL REFERENCES users(id),

    amount BIGINT NOT NULL,
    description TEXT,

    chargee_user_id INT NOT NULL REFERENCES users(id),
    charged_user_id INT NOT NULL REFERENCES users(id),
    charge_method user_expenses_charge_method NOT NULL,
    begin_charging_at TIMESTAMPTZ NOT NULL
); 

CREATE TABLE user_expense_installments (
    id SERIAL PRIMARY KEY,
    user_expense_id INT NOT NULL REFERENCES user_expenses(id),
    charged_at TIMESTAMPTZ NOT NULL,

    amount BIGINT NOT NULL
);
