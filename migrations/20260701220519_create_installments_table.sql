CREATE TABLE installments (
    id INT GENERATED ALWAYS AS IDENTITY,
    credit_card_id INT NOT NULL,
    description VARCHAR(255) NOT NULL,
    total_amount NUMERIC(12, 2) NOT NULL,
    installments_count SMALLINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_installment PRIMARY KEY (id),
    CONSTRAINT fk_installment_credit_card FOREIGN KEY (credit_card_id) REFERENCES credit_cards (id) ON DELETE RESTRICT,
    CONSTRAINT chk_positive_total_amount CHECK (total_amount > 0),
    CONSTRAINT chk_positive_installments_count CHECK (installments_count > 0)
);
