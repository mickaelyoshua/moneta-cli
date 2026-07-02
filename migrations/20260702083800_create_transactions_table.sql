CREATE TYPE transaction_status_enum AS ENUM ('pending', 'cleared');

CREATE TABLE transactions (
    id INT GENERATED ALWAYS AS IDENTITY,
    category_id INT NOT NULL,
    account_id INT,
    credit_card_id INT,
    installment_id INT,
    recurrence_id INT,
    
    transaction_type transaction_type_enum NOT NULL,
    amount NUMERIC(12, 2) NOT NULL,
    date DATE NOT NULL,
    description VARCHAR(255) NOT NULL,
    installment_number SMALLINT,
    status transaction_status_enum NOT NULL DEFAULT 'cleared',
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_transaction PRIMARY KEY (id),
    CONSTRAINT fk_transaction_category FOREIGN KEY (category_id) REFERENCES categories (id) ON DELETE RESTRICT,
    CONSTRAINT fk_transaction_account FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE RESTRICT,
    CONSTRAINT fk_transaction_credit_card FOREIGN KEY (credit_card_id) REFERENCES credit_cards (id) ON DELETE RESTRICT,
    CONSTRAINT fk_transaction_installment FOREIGN KEY (installment_id) REFERENCES installments (id) ON DELETE RESTRICT,
    CONSTRAINT fk_transaction_recurrence FOREIGN KEY (recurrence_id) REFERENCES recurrences (id) ON DELETE RESTRICT,

    CONSTRAINT chk_account_or_card CHECK (
        (account_id IS NOT NULL AND credit_card_id IS NULL) OR 
        (account_id IS NULL AND credit_card_id IS NOT NULL)
    ),

    CONSTRAINT chk_positive_amount CHECK (amount > 0)
);

CREATE INDEX idx_transactions_date ON transactions (date);
CREATE INDEX idx_transactions_account ON transactions (account_id);
CREATE INDEX idx_transactions_credit_card ON transactions (credit_card_id);
