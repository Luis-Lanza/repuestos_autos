use std::collections::HashSet;

use rusqlite::{Connection, Transaction, TransactionBehavior};

use crate::domain::catalog::{validate_category, validate_product};
use crate::infrastructure::sqlite::SqliteCatalogRepository;

const DEMO_CATEGORIES: [(&str, &str, usize); 8] = [
    ("Frenos", "FRN", 12),
    ("Suspension", "SUS", 12),
    ("Iluminacion", "ILU", 12),
    ("Motor", "MTR", 12),
    ("Transmision", "TRN", 12),
    ("Refrigeracion", "REF", 12),
    ("Neumaticos", "NEU", 13),
    ("Direccion", "DIR", 13),
];
const DEMO_DESCRIPTORS: [&str; 13] = [
    "Premium",
    "Standard",
    "Economy",
    "Heavy Duty",
    "Compact",
    "Performance",
    "Service",
    "Touring",
    "Original",
    "Pro",
    "Max",
    "Urban",
    "Fleet",
];

#[derive(Clone, Debug)]
pub(crate) struct BootstrapDemoCategory {
    pub(crate) name: String,
}

#[derive(Clone, Debug)]
pub(crate) struct BootstrapDemoProduct {
    pub(crate) category_index: usize,
    pub(crate) sku: String,
    pub(crate) name: String,
    pub(crate) list_price_centavos: i64,
    pub(crate) minimum_sale_price_centavos: i64,
    pub(crate) opening_quantity: i64,
}

#[derive(Clone, Debug)]
pub(crate) struct BootstrapDemoPlan {
    pub(crate) categories: Vec<BootstrapDemoCategory>,
    pub(crate) products: Vec<BootstrapDemoProduct>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BootstrapDemoEligibility {
    Pristine,
    Complete,
    Refused,
}

pub(crate) trait BootstrapDemoRepository {
    fn inspect_bootstrap(
        &self,
        transaction: &Transaction<'_>,
        plan: &BootstrapDemoPlan,
    ) -> rusqlite::Result<BootstrapDemoEligibility>;

    fn persist_bootstrap(
        &self,
        transaction: &Transaction<'_>,
        plan: &BootstrapDemoPlan,
    ) -> rusqlite::Result<()>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootstrapDemoOutcome {
    Loaded(BootstrapDemoSummary),
    AlreadyComplete(BootstrapDemoSummary),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BootstrapDemoSummary {
    pub categories_added: usize,
    pub products_added: usize,
    pub products_reactivated: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootstrapDemoError {
    InvalidPlan,
    Refused,
    Persistence,
}

pub fn bootstrap_demo_catalog(
    connection: &mut Connection,
) -> Result<BootstrapDemoOutcome, BootstrapDemoError> {
    let plan = build_demo_plan()?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| BootstrapDemoError::Persistence)?;
    let repository = SqliteCatalogRepository;
    let eligibility = repository
        .inspect_bootstrap(&transaction, &plan)
        .map_err(|_| BootstrapDemoError::Persistence)?;

    let outcome = match eligibility {
        BootstrapDemoEligibility::Complete => BootstrapDemoOutcome::AlreadyComplete(
            BootstrapDemoSummary {
                categories_added: 0,
                products_added: 0,
                products_reactivated: 0,
            },
        ),
        BootstrapDemoEligibility::Pristine => {
            repository
                .persist_bootstrap(&transaction, &plan)
                .map_err(|_| BootstrapDemoError::Persistence)?;
            BootstrapDemoOutcome::Loaded(BootstrapDemoSummary {
                categories_added: plan.categories.len(),
                products_added: plan.products.len(),
                products_reactivated: 1,
            })
        }
        BootstrapDemoEligibility::Refused => return Err(BootstrapDemoError::Refused),
    };

    transaction
        .commit()
        .map_err(|_| BootstrapDemoError::Persistence)?;
    Ok(outcome)
}

fn build_demo_plan() -> Result<BootstrapDemoPlan, BootstrapDemoError> {
    let categories = DEMO_CATEGORIES
        .iter()
        .map(|(name, _, _)| BootstrapDemoCategory {
            name: (*name).into(),
        })
        .collect::<Vec<_>>();
    for category in &categories {
        validate_category(&category.name, &[]).map_err(|_| BootstrapDemoError::InvalidPlan)?;
    }

    let mut products = Vec::with_capacity(98);
    let mut skus = HashSet::new();
    let mut names = HashSet::new();
    for (category_index, (category_name, code, product_count)) in DEMO_CATEGORIES.iter().enumerate()
    {
        let category_base = 3_200_i64
            .checked_add(
                (category_index as i64)
                    .checked_mul(750)
                    .ok_or(BootstrapDemoError::InvalidPlan)?,
            )
            .ok_or(BootstrapDemoError::InvalidPlan)?;
        for product_index in 0..*product_count {
            let descriptor = DEMO_DESCRIPTORS[product_index];
            let sku = format!("{code}-DEMO-{:03}", product_index + 1);
            let name = format!("{category_name} {descriptor} demo part");
            if !skus.insert(sku.to_lowercase()) || !names.insert(name.to_lowercase()) {
                return Err(BootstrapDemoError::InvalidPlan);
            }
            let list_price_centavos = category_base
                .checked_add(
                    (product_index as i64)
                        .checked_mul(275)
                        .ok_or(BootstrapDemoError::InvalidPlan)?,
                )
                .ok_or(BootstrapDemoError::InvalidPlan)?;
            let minimum_sale_price_centavos = list_price_centavos
                .checked_sub(350)
                .ok_or(BootstrapDemoError::InvalidPlan)?;
            let opening_quantity = 3_i64
                .checked_add(((category_index * 3 + product_index) % 8) as i64)
                .ok_or(BootstrapDemoError::InvalidPlan)?;
            validate_product(
                &sku,
                &name,
                list_price_centavos,
                minimum_sale_price_centavos,
                opening_quantity,
                &[],
                &[],
            )
            .map_err(|_| BootstrapDemoError::InvalidPlan)?;
            products.push(BootstrapDemoProduct {
                category_index,
                sku,
                name,
                list_price_centavos,
                minimum_sale_price_centavos,
                opening_quantity,
            });
        }
    }

    if products.len() != 98 {
        return Err(BootstrapDemoError::InvalidPlan);
    }
    Ok(BootstrapDemoPlan {
        categories,
        products,
    })
}
