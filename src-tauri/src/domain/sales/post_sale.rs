use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaleCorrectionState {
    Confirmed,
    Cancelled,
    NotConfirmed,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestedReturnLine {
    pub sale_line_id: i64,
    pub quantity: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OriginalSaleLine {
    pub sale_line_id: i64,
    pub product_id: i64,
    pub sold_quantity: i64,
    pub returned_quantity: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnPlanLine {
    pub sale_line_id: i64,
    pub product_id: i64,
    pub quantity: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnPlan {
    pub lines: Vec<ReturnPlanLine>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CancellationPlanLine {
    pub sale_line_id: i64,
    pub product_id: i64,
    pub restored_quantity: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CancellationPlan {
    pub reason: String,
    pub lines: Vec<CancellationPlanLine>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostSaleDomainError {
    EmptyReturn,
    DuplicateSaleLine,
    SaleNotFound,
    SaleLineNotFound,
    InvalidQuantity,
    QuantityExceedsRemaining,
    SaleNotConfirmed,
    SaleCancelled,
    CancellationAlreadyRecorded,
    CancellationReasonRequired,
    InvalidOriginalQuantity,
}

pub fn plan_return(
    state: SaleCorrectionState,
    originals: &[OriginalSaleLine],
    requested: &[RequestedReturnLine],
) -> Result<ReturnPlan, PostSaleDomainError> {
    match state {
        SaleCorrectionState::Confirmed => (),
        SaleCorrectionState::Cancelled => return Err(PostSaleDomainError::SaleCancelled),
        SaleCorrectionState::NotConfirmed => return Err(PostSaleDomainError::SaleNotConfirmed),
    }
    if requested.is_empty() {
        return Err(PostSaleDomainError::EmptyReturn);
    }
    let mut ids = HashSet::with_capacity(requested.len());
    requested
        .iter()
        .map(|line| {
            if line.sale_line_id <= 0 || line.quantity <= 0 {
                return Err(PostSaleDomainError::InvalidQuantity);
            }
            if !ids.insert(line.sale_line_id) {
                return Err(PostSaleDomainError::DuplicateSaleLine);
            }
            let original = originals
                .iter()
                .find(|item| item.sale_line_id == line.sale_line_id)
                .ok_or(PostSaleDomainError::SaleLineNotFound)?;
            if line.quantity > remaining(original)? {
                return Err(PostSaleDomainError::QuantityExceedsRemaining);
            }
            Ok(ReturnPlanLine {
                sale_line_id: original.sale_line_id,
                product_id: original.product_id,
                quantity: line.quantity,
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|lines| ReturnPlan { lines })
}

pub fn plan_cancellation(
    state: SaleCorrectionState,
    originals: &[OriginalSaleLine],
    reason: &str,
) -> Result<CancellationPlan, PostSaleDomainError> {
    match state {
        SaleCorrectionState::Confirmed => (),
        SaleCorrectionState::Cancelled => {
            return Err(PostSaleDomainError::CancellationAlreadyRecorded)
        }
        SaleCorrectionState::NotConfirmed => return Err(PostSaleDomainError::SaleNotConfirmed),
    }
    let reason = reason.trim();
    if reason.is_empty() {
        return Err(PostSaleDomainError::CancellationReasonRequired);
    }
    originals
        .iter()
        .map(|line| {
            Ok(CancellationPlanLine {
                sale_line_id: line.sale_line_id,
                product_id: line.product_id,
                restored_quantity: remaining(line)?,
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|lines| CancellationPlan {
            reason: reason.into(),
            lines,
        })
}

fn remaining(line: &OriginalSaleLine) -> Result<i64, PostSaleDomainError> {
    if line.sale_line_id <= 0
        || line.product_id <= 0
        || line.sold_quantity <= 0
        || line.returned_quantity < 0
    {
        return Err(PostSaleDomainError::InvalidOriginalQuantity);
    }
    let remaining = line
        .sold_quantity
        .checked_sub(line.returned_quantity)
        .ok_or(PostSaleDomainError::InvalidOriginalQuantity)?;
    if remaining < 0 {
        return Err(PostSaleDomainError::InvalidOriginalQuantity);
    }
    Ok(remaining)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fact(id: i64, product: i64, sold: i64, returned: i64) -> OriginalSaleLine {
        OriginalSaleLine {
            sale_line_id: id,
            product_id: product,
            sold_quantity: sold,
            returned_quantity: returned,
        }
    }
    fn request(id: i64, quantity: i64) -> RequestedReturnLine {
        RequestedReturnLine {
            sale_line_id: id,
            quantity,
        }
    }

    #[test]
    fn return_uses_line_identity_and_rejects_invalid_requests() {
        let facts = [fact(1, 7, 4, 1), fact(2, 7, 2, 0)];
        assert_eq!(
            plan_return(
                SaleCorrectionState::Confirmed,
                &facts,
                &[request(2, 2), request(1, 3)]
            )
            .unwrap()
            .lines,
            vec![
                ReturnPlanLine {
                    sale_line_id: 2,
                    product_id: 7,
                    quantity: 2
                },
                ReturnPlanLine {
                    sale_line_id: 1,
                    product_id: 7,
                    quantity: 3
                }
            ]
        );
        for lines in [
            vec![],
            vec![request(1, 0)],
            vec![request(3, 1)],
            vec![request(1, 4)],
            vec![request(1, 1), request(1, 1)],
        ] {
            assert!(plan_return(SaleCorrectionState::Confirmed, &facts, &lines).is_err());
        }
        assert_eq!(
            plan_return(SaleCorrectionState::Confirmed, &facts, &[request(1, -1)]),
            Err(PostSaleDomainError::InvalidQuantity)
        );
        assert_eq!(
            plan_return(SaleCorrectionState::Cancelled, &facts, &[request(1, 1)]),
            Err(PostSaleDomainError::SaleCancelled)
        );
        assert_eq!(
            plan_return(SaleCorrectionState::NotConfirmed, &facts, &[request(1, 1)]),
            Err(PostSaleDomainError::SaleNotConfirmed)
        );
    }

    #[test]
    fn cancellation_normalizes_reason_and_derives_residuals() {
        let facts = [fact(1, 7, 4, 1), fact(2, 8, 2, 2)];
        assert_eq!(
            plan_cancellation(SaleCorrectionState::Confirmed, &facts, " x ").unwrap(),
            CancellationPlan {
                reason: "x".into(),
                lines: vec![
                    CancellationPlanLine {
                        sale_line_id: 1,
                        product_id: 7,
                        restored_quantity: 3
                    },
                    CancellationPlanLine {
                        sale_line_id: 2,
                        product_id: 8,
                        restored_quantity: 0
                    }
                ]
            }
        );
        assert_eq!(
            plan_cancellation(SaleCorrectionState::Confirmed, &[fact(1, 7, 4, 0)], "x")
                .unwrap()
                .lines[0]
                .restored_quantity,
            4
        );
        assert_eq!(
            plan_cancellation(SaleCorrectionState::Confirmed, &[fact(1, 1, 3, 4)], "x"),
            Err(PostSaleDomainError::InvalidOriginalQuantity)
        );
        assert_eq!(
            plan_cancellation(SaleCorrectionState::Confirmed, &facts, " "),
            Err(PostSaleDomainError::CancellationReasonRequired)
        );
        assert_eq!(
            plan_cancellation(SaleCorrectionState::Cancelled, &facts, "x"),
            Err(PostSaleDomainError::CancellationAlreadyRecorded)
        );
        assert_eq!(
            plan_cancellation(SaleCorrectionState::NotConfirmed, &facts, "x"),
            Err(PostSaleDomainError::SaleNotConfirmed)
        );
        assert_eq!(
            plan_cancellation(
                SaleCorrectionState::Confirmed,
                &[fact(1, 1, i64::MIN, 0)],
                "x"
            ),
            Err(PostSaleDomainError::InvalidOriginalQuantity)
        );
    }
}
