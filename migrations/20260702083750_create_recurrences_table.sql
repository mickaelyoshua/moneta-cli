CREATE TABLE recurrences (
    id INT GENERATED ALWAYS AS IDENTITY,
    category_id INT NOT NULL,
    account_id INT,
    credit_card_id INT,
    
    transaction_type VARCHAR(20) NOT NULL,
    amount NUMERIC(12, 2) NOT NULL,
    description VARCHAR(255) NOT NULL,
    frequency VARCHAR(20) NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE,
    last_processed_date DATE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_recurrence PRIMARY KEY (id),
    CONSTRAINT fk_recurrence_category FOREIGN KEY (category_id) REFERENCES categories (id),
    CONSTRAINT fk_recurrence_account FOREIGN KEY (account_id) REFERENCES accounts (id),
    CONSTRAINT fk_recurrence_credit_card FOREIGN KEY (credit_card_id) REFERENCES credit_cards (id),

    CONSTRAINT chk_recurrence_type CHECK (transaction_type IN ('income', 'expense', 'transfer')),
    CONSTRAINT chk_recurrence_account_or_card CHECK (
        (account_id IS NOT NULL AND credit_card_id IS NULL) OR 
        (account_id IS NULL AND credit_card_id IS NOT NULL)
    ),
    CONSTRAINT chk_recurrence_frequency CHECK (frequency IN ('daily', 'weekly', 'monthly', 'yearly')),
    CONSTRAINT chk_recurrence_positive_amount CHECK (amount > 0)
);
