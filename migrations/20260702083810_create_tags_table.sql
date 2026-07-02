CREATE TABLE tags (
    id INT GENERATED ALWAYS AS IDENTITY,
    name VARCHAR(50) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_tag PRIMARY KEY (id)
);

CREATE TABLE transaction_tags (
    transaction_id INT NOT NULL,
    tag_id INT NOT NULL,
    
    CONSTRAINT pk_transaction_tag PRIMARY KEY (transaction_id, tag_id),
    CONSTRAINT fk_tt_transaction FOREIGN KEY (transaction_id) REFERENCES transactions (id) ON DELETE CASCADE,
    CONSTRAINT fk_tt_tag FOREIGN KEY (tag_id) REFERENCES tags (id) ON DELETE CASCADE
);

CREATE INDEX idx_transaction_tags_tag ON transaction_tags (tag_id);
