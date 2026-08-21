use crate::domain::{MoneyCentavos, Quantity};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SaleLine {
    product_id: i64,
    quantity: Quantity,
    negotiated_unit_price: MoneyCentavos,
    minimum_unit_price_snapshot: MoneyCentavos,
    total: MoneyCentavos,
}

impl SaleLine {
    pub fn new(
        product_id: i64,
        quantity: Quantity,
        negotiated_unit_price: MoneyCentavos,
        minimum_unit_price_snapshot: MoneyCentavos,
    ) -> Result<Self, &'static str> {
        if negotiated_unit_price.value() < minimum_unit_price_snapshot.value() {
            return Err("negotiated price is below the current minimum");
        }
        let total = negotiated_unit_price.checked_multiply(quantity.value())?;
        Ok(Self {
            product_id,
            quantity,
            negotiated_unit_price,
            minimum_unit_price_snapshot,
            total,
        })
    }

    pub fn product_id(self) -> i64 {
        self.product_id
    }

    pub fn quantity(self) -> Quantity {
        self.quantity
    }

    pub fn negotiated_unit_price(self) -> MoneyCentavos {
        self.negotiated_unit_price
    }

    pub fn total(self) -> MoneyCentavos {
        self.total
    }

    pub fn minimum_unit_price_snapshot(self) -> MoneyCentavos {
        self.minimum_unit_price_snapshot
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Payment {
    Cash {
        amount_applied: MoneyCentavos,
        amount_tendered: MoneyCentavos,
        change_given: MoneyCentavos,
    },
    Qr {
        amount_applied: MoneyCentavos,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Sale {
    lines: Vec<SaleLine>,
    payments: Vec<Payment>,
    total: MoneyCentavos,
}

impl Sale {
    pub fn new(lines: Vec<SaleLine>, payments: Vec<Payment>) -> Result<Self, &'static str> {
        if lines.is_empty() {
            return Err("sale must include at least one line");
        }
        let total = lines.iter().try_fold(MoneyCentavos::new(0)?, |sum, line| {
            sum.checked_add(line.total())
        })?;
        let applied = payments
            .iter()
            .try_fold(MoneyCentavos::new(0)?, |sum, payment| {
                sum.checked_add(payment.amount_applied())
            })?;
        if applied != total {
            return Err("applied payments must equal the sale total");
        }
        Ok(Self {
            lines,
            payments,
            total,
        })
    }

    pub fn total(&self) -> MoneyCentavos {
        self.total
    }

    pub fn lines(&self) -> &[SaleLine] {
        &self.lines
    }

    pub fn payments(&self) -> &[Payment] {
        &self.payments
    }
}

impl Payment {
    pub fn cash(
        amount_applied: MoneyCentavos,
        amount_tendered: MoneyCentavos,
        change_given: MoneyCentavos,
    ) -> Result<Self, &'static str> {
        if amount_tendered.value().checked_sub(amount_applied.value()) != Some(change_given.value())
        {
            return Err("cash tender and change are inconsistent");
        }
        Ok(Self::Cash {
            amount_applied,
            amount_tendered,
            change_given,
        })
    }

    pub fn qr(amount_applied: MoneyCentavos) -> Self {
        Self::Qr { amount_applied }
    }

    pub fn amount_applied(self) -> MoneyCentavos {
        match self {
            Self::Cash { amount_applied, .. } | Self::Qr { amount_applied } => amount_applied,
        }
    }
}
