use serde::Deserialize;

use crate::application::catalog;

#[derive(Debug, Deserialize)]
pub struct SearchProductsRequest {
    pub query: String,
}

pub use catalog::ProductSearchResult;

pub fn search_products(
    connection: &rusqlite::Connection,
    request: SearchProductsRequest,
) -> Result<Vec<ProductSearchResult>, String> {
    catalog::search_active_products(connection, &request.query)
        .map_err(|_| "persistence_failure".into())
}
