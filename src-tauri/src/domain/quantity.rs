#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quantity(i64);

impl Quantity {
    pub fn new(value: i64) -> Result<Self, &'static str> {
        if value <= 0 {
            return Err("quantity must be positive");
        }
        Ok(Self(value))
    }

    pub fn value(self) -> i64 {
        self.0
    }
}
