CREATE UNIQUE INDEX sale_lines_identity_idx ON sale_lines (id, sale_id, product_id);

CREATE TABLE post_sale_requests (
    id INTEGER PRIMARY KEY,
    request_id TEXT NOT NULL UNIQUE CHECK (trim(request_id) <> ''),
    operation_kind TEXT NOT NULL CHECK (operation_kind IN ('return', 'cancellation')),
    sale_id INTEGER NOT NULL REFERENCES sales(id),
    payload_version INTEGER NOT NULL CHECK (payload_version = 1),
    canonical_payload BLOB NOT NULL,
    payload_sha256 TEXT NOT NULL CHECK (length(payload_sha256) = 64),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (id, sale_id, operation_kind)
);

CREATE TABLE sale_returns (
    id INTEGER PRIMARY KEY,
    sale_id INTEGER NOT NULL,
    operation_kind TEXT NOT NULL DEFAULT 'return' CHECK (operation_kind = 'return'),
    occurred_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (id, sale_id),
    FOREIGN KEY (id, sale_id, operation_kind)
        REFERENCES post_sale_requests(id, sale_id, operation_kind)
);

CREATE TABLE sale_return_lines (
    return_id INTEGER NOT NULL,
    sale_id INTEGER NOT NULL,
    sale_line_id INTEGER NOT NULL,
    product_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    movement_id INTEGER NOT NULL UNIQUE REFERENCES inventory_movements(id),
    PRIMARY KEY (return_id, sale_line_id),
    FOREIGN KEY (return_id, sale_id) REFERENCES sale_returns(id, sale_id),
    FOREIGN KEY (sale_line_id, sale_id, product_id)
        REFERENCES sale_lines(id, sale_id, product_id)
);

CREATE TABLE sale_cancellations (
    id INTEGER PRIMARY KEY,
    sale_id INTEGER NOT NULL UNIQUE,
    operation_kind TEXT NOT NULL DEFAULT 'cancellation' CHECK (operation_kind = 'cancellation'),
    reason TEXT NOT NULL CHECK (trim(reason) <> ''),
    occurred_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (id, sale_id),
    FOREIGN KEY (id, sale_id, operation_kind)
        REFERENCES post_sale_requests(id, sale_id, operation_kind)
);

CREATE TABLE sale_cancellation_lines (
    cancellation_id INTEGER NOT NULL,
    sale_id INTEGER NOT NULL,
    sale_line_id INTEGER NOT NULL,
    product_id INTEGER NOT NULL,
    restored_quantity INTEGER NOT NULL CHECK (restored_quantity >= 0),
    movement_id INTEGER UNIQUE REFERENCES inventory_movements(id),
    PRIMARY KEY (cancellation_id, sale_line_id),
    FOREIGN KEY (cancellation_id, sale_id) REFERENCES sale_cancellations(id, sale_id),
    FOREIGN KEY (sale_line_id, sale_id, product_id)
        REFERENCES sale_lines(id, sale_id, product_id),
    CHECK ((restored_quantity = 0 AND movement_id IS NULL)
        OR (restored_quantity > 0 AND movement_id IS NOT NULL))
);

CREATE INDEX post_sale_requests_sale_created_idx ON post_sale_requests (sale_id, created_at, id);
CREATE INDEX sale_return_lines_sale_line_idx ON sale_return_lines (sale_line_id, return_id);
CREATE INDEX sale_cancellation_lines_sale_line_idx ON sale_cancellation_lines (sale_line_id);

CREATE TRIGGER post_sale_requests_immutable_update BEFORE UPDATE ON post_sale_requests BEGIN SELECT RAISE(ABORT, 'post-sale facts are immutable'); END;
CREATE TRIGGER post_sale_requests_immutable_delete BEFORE DELETE ON post_sale_requests BEGIN SELECT RAISE(ABORT, 'post-sale facts are immutable'); END;
CREATE TRIGGER sale_returns_immutable_update BEFORE UPDATE ON sale_returns BEGIN SELECT RAISE(ABORT, 'post-sale facts are immutable'); END;
CREATE TRIGGER sale_returns_immutable_delete BEFORE DELETE ON sale_returns BEGIN SELECT RAISE(ABORT, 'post-sale facts are immutable'); END;
CREATE TRIGGER sale_return_lines_immutable_update BEFORE UPDATE ON sale_return_lines BEGIN SELECT RAISE(ABORT, 'post-sale facts are immutable'); END;
CREATE TRIGGER sale_return_lines_immutable_delete BEFORE DELETE ON sale_return_lines BEGIN SELECT RAISE(ABORT, 'post-sale facts are immutable'); END;
CREATE TRIGGER sale_cancellations_immutable_update BEFORE UPDATE ON sale_cancellations BEGIN SELECT RAISE(ABORT, 'post-sale facts are immutable'); END;
CREATE TRIGGER sale_cancellations_immutable_delete BEFORE DELETE ON sale_cancellations BEGIN SELECT RAISE(ABORT, 'post-sale facts are immutable'); END;
CREATE TRIGGER sale_cancellation_lines_immutable_update BEFORE UPDATE ON sale_cancellation_lines BEGIN SELECT RAISE(ABORT, 'post-sale facts are immutable'); END;
CREATE TRIGGER sale_cancellation_lines_immutable_delete BEFORE DELETE ON sale_cancellation_lines BEGIN SELECT RAISE(ABORT, 'post-sale facts are immutable'); END;

CREATE TRIGGER sale_return_lines_validate_insert BEFORE INSERT ON sale_return_lines BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1 FROM inventory_movements
        WHERE id = NEW.movement_id AND product_id = NEW.product_id AND sale_id = NEW.sale_id
          AND sale_line_id = NEW.sale_line_id AND movement_type = 'return'
          AND quantity_delta = NEW.quantity AND quantity_delta > 0
    ) THEN RAISE(ABORT, 'return movement must match its original sale line') END;
    SELECT CASE WHEN (
        NEW.quantity + COALESCE((SELECT SUM(quantity) FROM sale_return_lines WHERE sale_line_id = NEW.sale_line_id), 0)
        + COALESCE((SELECT restored_quantity FROM sale_cancellation_lines WHERE sale_line_id = NEW.sale_line_id), 0)
    ) > (SELECT quantity FROM sale_lines WHERE id = NEW.sale_line_id)
    THEN RAISE(ABORT, 'restoration exceeds sold quantity') END;
END;

CREATE TRIGGER sale_cancellation_lines_validate_insert BEFORE INSERT ON sale_cancellation_lines BEGIN
    SELECT CASE WHEN NEW.restored_quantity > 0 AND NOT EXISTS (
        SELECT 1 FROM inventory_movements
        WHERE id = NEW.movement_id AND product_id = NEW.product_id AND sale_id = NEW.sale_id
          AND sale_line_id = NEW.sale_line_id AND movement_type = 'cancellation'
          AND quantity_delta = NEW.restored_quantity AND quantity_delta > 0
    ) THEN RAISE(ABORT, 'cancellation movement must match its original sale line') END;
    SELECT CASE WHEN NEW.restored_quantity = 0 AND NEW.movement_id IS NOT NULL
        THEN RAISE(ABORT, 'zero cancellation quantity cannot have a movement') END;
    SELECT CASE WHEN NEW.restored_quantity + COALESCE((SELECT SUM(quantity) FROM sale_return_lines WHERE sale_line_id = NEW.sale_line_id), 0)
        > (SELECT quantity FROM sale_lines WHERE id = NEW.sale_line_id)
    THEN RAISE(ABORT, 'restoration exceeds sold quantity') END;
END;
