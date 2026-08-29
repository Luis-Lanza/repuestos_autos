CREATE INDEX sales_confirmed_history_idx ON sales (confirmed_at DESC, id DESC) WHERE status = 'confirmed';
