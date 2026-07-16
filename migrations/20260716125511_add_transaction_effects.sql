CREATE OR REPLACE VIEW v_transaction_totals AS
SELECT id as transaction_id,
       CASE WHEN transaction_type = 'income' THEN amount ELSE -amount END as account_effect,
       CASE WHEN transaction_type = 'income' THEN -amount ELSE amount END as expense_effect
FROM transactions;
