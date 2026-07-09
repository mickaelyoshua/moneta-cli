CREATE TYPE budget_period AS ENUM ('weekly', 'monthly', 'yearly');

CREATE TABLE budgets (
    id INT GENERATED ALWAYS AS IDENTITY,
    category_id INT,
    tag_id INT,
    amount_limit NUMERIC(12, 2) NOT NULL,
    period budget_period NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_budget PRIMARY KEY (id),
    CONSTRAINT fk_budget_category FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE,
    CONSTRAINT fk_budget_tag FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
    CONSTRAINT chk_budget_target CHECK (category_id IS NOT NULL OR tag_id IS NOT NULL)
);

CREATE UNIQUE INDEX idx_budgets_category ON budgets (category_id, period) WHERE category_id IS NOT NULL;
CREATE UNIQUE INDEX idx_budgets_tag ON budgets (tag_id, period) WHERE tag_id IS NOT NULL;
