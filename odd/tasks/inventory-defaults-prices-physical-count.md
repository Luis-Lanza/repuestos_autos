# Inventory defaults, optional prices, and physical-count authorization

## Objective
Keep Inventory on All Stock regardless of alert entry, preserve current product prices when Stock Entry prices are omitted, and require the shared Catalog password before a physical count mutates stock.

## Decisions
- Inventory always opens with All Stock; alerts remain informational and never select the Alerts filter automatically.
- Purchase, sale, and minimum-sale prices are optional in Stock Entry; each omitted value keeps the current persisted product value.
- Physical count uses the existing device-local Catalog password verifier. It creates no Inventory credential and immediately follows future Catalog password changes.
- Password verification is enforced in Rust before the physical-count application operation; the modal is not the security boundary.

## Tasks
- [~] I1 Keep the Inventory entry state on All Stock and cover alert navigation. — Implementation and independent checks passed; awaiting explicit local-commit authorization.
- [ ] I2 Make all Stock Entry price inputs optional while preserving omitted persisted values.
- [ ] I3 Require shared Catalog-password verification for physical-count confirmation end to end.
- [ ] I4 Run focused Rust/frontend checks and record Windows smoke cases.

## Acceptance criteria
- Inventory opens on All Stock even when stock alerts exist or prompted navigation originated from an alert.
- Any subset of Stock Entry prices can be omitted; omitted values remain unchanged and supplied values retain validation.
- Physical count cannot mutate stock without the current Catalog password; wrong/missing credentials are rejected without mutation.
- All user-visible password states are accessible and the shared password change takes effect without duplication.
