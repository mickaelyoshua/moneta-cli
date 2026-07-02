CREATE TYPE account_type_enum AS ENUM ('checking', 'savings');

CREATE TABLE accounts (
    id INT GENERATED ALWAYS AS IDENTITY,
    name VARCHAR(100) NOT NULL,
    account_type account_type_enum NOT NULL DEFAULT 'checking',
    has_debit_card BOOLEAN NOT NULL DEFAULT TRUE,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

	CONSTRAINT pk_account PRIMARY KEY (id)
);
