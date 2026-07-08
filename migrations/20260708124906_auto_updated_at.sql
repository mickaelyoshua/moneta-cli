CREATE EXTENSION IF NOT EXISTS moddatetime;

CREATE TRIGGER update_categories_updated_at BEFORE UPDATE ON categories FOR EACH ROW EXECUTE PROCEDURE moddatetime(updated_at);
CREATE TRIGGER update_accounts_updated_at BEFORE UPDATE ON accounts FOR EACH ROW EXECUTE PROCEDURE moddatetime(updated_at);
CREATE TRIGGER update_credit_cards_updated_at BEFORE UPDATE ON credit_cards FOR EACH ROW EXECUTE PROCEDURE moddatetime(updated_at);
CREATE TRIGGER update_invoices_updated_at BEFORE UPDATE ON invoices FOR EACH ROW EXECUTE PROCEDURE moddatetime(updated_at);
CREATE TRIGGER update_installments_updated_at BEFORE UPDATE ON installments FOR EACH ROW EXECUTE PROCEDURE moddatetime(updated_at);
CREATE TRIGGER update_recurrences_updated_at BEFORE UPDATE ON recurrences FOR EACH ROW EXECUTE PROCEDURE moddatetime(updated_at);
CREATE TRIGGER update_transactions_updated_at BEFORE UPDATE ON transactions FOR EACH ROW EXECUTE PROCEDURE moddatetime(updated_at);
CREATE TRIGGER update_tags_updated_at BEFORE UPDATE ON tags FOR EACH ROW EXECUTE PROCEDURE moddatetime(updated_at);
