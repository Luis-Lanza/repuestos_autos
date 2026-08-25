use serde::Serialize;

use crate::domain::{MoneyCentavos, Quantity};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SaleLine {
    product_id: i64,
    quantity: Quantity,
    unit_price: MoneyCentavos,
    minimum_unit_price_snapshot: MoneyCentavos,
    total: MoneyCentavos,
}

impl SaleLine {
    pub fn priced(
        product_id: i64,
        quantity: Quantity,
        unit_price: MoneyCentavos,
    ) -> Result<Self, SaleError> {
        let total = unit_price
            .checked_multiply(quantity.value())
            .map_err(|_| SaleError::MoneyOverflow)?;
        Ok(Self {
            product_id,
            quantity,
            unit_price,
            minimum_unit_price_snapshot: unit_price,
            total,
        })
    }

    // Retained until the repository is migrated to resolve catalog prices itself.
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
            unit_price: negotiated_unit_price,
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

    pub fn unit_price(self) -> MoneyCentavos {
        self.unit_price
    }

    // Retained for legacy SQLite mapping until PR 3 changes the persistence contract.
    pub fn negotiated_unit_price(self) -> MoneyCentavos {
        self.unit_price()
    }

    pub fn total(self) -> MoneyCentavos {
        self.total
    }

    pub fn minimum_unit_price_snapshot(self) -> MoneyCentavos {
        self.minimum_unit_price_snapshot
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaleError {
    EmptyLines,
    MoneyOverflow,
    AppliedPaymentsDoNotEqualTotal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaymentInput {
    pub amount_tendered: Option<MoneyCentavos>,
    pub qr_applied: Option<MoneyCentavos>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaymentError {
    QrExceedsTotal,
    CashTenderRequired,
    InsufficientCashTender,
    UnexpectedCashTender,
    MoneyOverflow,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentBreakdown {
    payments: Vec<Payment>,
}

impl PaymentBreakdown {
    pub fn derive(total: MoneyCentavos, input: PaymentInput) -> Result<Self, PaymentError> {
        let qr_applied = input
            .qr_applied
            .unwrap_or(MoneyCentavos::new(0).map_err(|_| PaymentError::MoneyOverflow)?);
        if qr_applied.value() > total.value() {
            return Err(PaymentError::QrExceedsTotal);
        }

        let cash_applied = subtract(total, qr_applied)?;
        let mut payments = Vec::with_capacity(2);
        if qr_applied.value() > 0 {
            payments.push(Payment::qr(qr_applied));
        }

        if cash_applied.value() == 0 {
            if input
                .amount_tendered
                .is_some_and(|tendered| tendered.value() > 0)
            {
                return Err(PaymentError::UnexpectedCashTender);
            }
            return Ok(Self { payments });
        }

        let amount_tendered = input
            .amount_tendered
            .ok_or(PaymentError::CashTenderRequired)?;
        if amount_tendered.value() < cash_applied.value() {
            return Err(PaymentError::InsufficientCashTender);
        }
        let change_given = subtract(amount_tendered, cash_applied)?;
        payments.push(Payment::Cash {
            amount_applied: cash_applied,
            amount_tendered,
            change_given,
        });

        Ok(Self { payments })
    }

    pub fn payments(&self) -> &[Payment] {
        &self.payments
    }
}

fn subtract(
    minuend: MoneyCentavos,
    subtrahend: MoneyCentavos,
) -> Result<MoneyCentavos, PaymentError> {
    minuend
        .value()
        .checked_sub(subtrahend.value())
        .ok_or(PaymentError::MoneyOverflow)
        .and_then(|value| MoneyCentavos::new(value).map_err(|_| PaymentError::MoneyOverflow))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "method", rename_all = "lowercase")]
pub enum Payment {
    Cash {
        #[serde(rename = "amount_applied_centavos")]
        amount_applied: MoneyCentavos,
        #[serde(rename = "amount_tendered_centavos")]
        amount_tendered: MoneyCentavos,
        #[serde(rename = "change_given_centavos")]
        change_given: MoneyCentavos,
    },
    Qr {
        #[serde(rename = "amount_applied_centavos")]
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
