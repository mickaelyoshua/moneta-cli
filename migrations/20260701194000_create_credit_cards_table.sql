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
