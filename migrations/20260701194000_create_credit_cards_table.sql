CREATE TABLE credit_cards (
    id INT GENERATED ALWAYS AS IDENTITY,
    account_id INT NOT NULL,
    name VARCHAR(100) NOT NULL,
    credit_limit NUMERIC(12, 2) NOT NULL,
    billing_day SMALLINT NOT NULL,
    due_day SMALLINT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_credit_card PRIMARY KEY (id),
    CONSTRAINT fk_credit_card_account FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE RESTRICT,
    
    CONSTRAINT chk_positive_credit_limit CHECK (credit_limit >= 0),
    CONSTRAINT chk_billing_day CHECK (billing_day BETWEEN 1 AND 28),
    CONSTRAINT chk_due_day CHECK (due_day BETWEEN 1 AND 28)
);

CREATE TYPE invoice_status_enum AS ENUM ('open', 'closed', 'paid');

CREATE TABLE invoices (
    id INT GENERATED ALWAYS AS IDENTITY,
    credit_card_id INT NOT NULL,
    month SMALLINT NOT NULL,
    year SMALLINT NOT NULL,
    status invoice_status_enum NOT NULL DEFAULT 'open',
    closing_amount NUMERIC(12, 2) NULL,
    due_date DATE NOT NULL,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_invoice PRIMARY KEY (id),
    CONSTRAINT fk_invoice_credit_card FOREIGN KEY (credit_card_id) REFERENCES credit_cards (id) ON DELETE CASCADE,
    CONSTRAINT uq_invoice_month_year UNIQUE (credit_card_id, month, year),
    CONSTRAINT chk_invoice_month CHECK (month BETWEEN 1 AND 12)
);
