-- Catalog prices are JavaScript-safe integer centavos and must remain explicit.
CREATE TRIGGER products_validate_price_insert
BEFORE INSERT ON products
WHEN NEW.list_price_centavos IS NULL
  OR NEW.list_price_centavos <= 0
  OR NEW.list_price_centavos > 9007199254740991
  OR NEW.list_price_centavos = 9223372036854775807
  OR NEW.minimum_unit_price_centavos IS NULL
  OR NEW.minimum_unit_price_centavos <= 0
  OR NEW.minimum_unit_price_centavos > 9007199254740991
  OR NEW.minimum_unit_price_centavos = 9223372036854775807
  OR NEW.minimum_unit_price_centavos > NEW.list_price_centavos
BEGIN
    SELECT RAISE(ABORT, 'catalog price is outside the safe integer range');
END;

CREATE TRIGGER products_validate_price_update
BEFORE UPDATE OF list_price_centavos, minimum_unit_price_centavos ON products
WHEN NEW.list_price_centavos IS NULL
  OR NEW.list_price_centavos <= 0
  OR NEW.list_price_centavos > 9007199254740991
  OR NEW.list_price_centavos = 9223372036854775807
  OR NEW.minimum_unit_price_centavos IS NULL
  OR NEW.minimum_unit_price_centavos <= 0
  OR NEW.minimum_unit_price_centavos > 9007199254740991
  OR NEW.minimum_unit_price_centavos = 9223372036854775807
  OR NEW.minimum_unit_price_centavos > NEW.list_price_centavos
BEGIN
    SELECT RAISE(ABORT, 'catalog price is outside the safe integer range');
END;
