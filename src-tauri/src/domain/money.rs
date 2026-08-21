use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct MoneyCentavos(i64);

impl MoneyCentavos {
    pub fn new(value: i64) -> Result<Self, &'static str> {
        if value < 0 {
            return Err("money cannot be negative");
        }
        Ok(Self(value))
    }

    pub fn checked_add(self, other: Self) -> Result<Self, &'static str> {
        self.0
            .checked_add(other.0)
            .ok_or("money total overflow")
            .and_then(Self::new)
    }

    pub fn checked_multiply(self, quantity: i64) -> Result<Self, &'static str> {
        self.0
            .checked_mul(quantity)
            .ok_or("money total overflow")
            .and_then(Self::new)
    }

    pub fn value(self) -> i64 {
        self.0
    }
}
