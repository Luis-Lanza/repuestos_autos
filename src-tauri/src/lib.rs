pub mod application;
pub mod domain;
pub mod infrastructure;

pub mod catalog {
    pub use crate::application::catalog::{search_active_products, ProductSearchResult};
    pub use crate::infrastructure::sqlite::open_seeded_catalog;
}
