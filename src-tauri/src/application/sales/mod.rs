mod application_contract;
pub mod confirm_sale;
pub mod history;
pub mod post_sale;
mod repository;

pub use application_contract::{
    ApplicationConfirmSaleRequest, ApplicationRequestedLine, ConfirmSaleUseCase,
};
pub use confirm_sale::{
    confirm_sale, ConfirmSaleRequest, PersistedLine, PersistedSaleSummary, RequestedLine,
};
pub use history::{
    HistoricalCancellation, HistoricalCancellationLine, HistoricalLine, HistoricalPayment,
    HistoricalReturn, HistoricalReturnLine, HistoryError, HistoryRange, PaymentMethod,
    SaleHistoryDetail, SaleHistoryDetailReader, SaleHistoryPage, SaleHistorySummary,
    SaleHistorySummaryReader,
};
pub use post_sale::{
    CancelSaleRequest, CancellationResult, CreateReturnRequest, PersistedRequest, PostSaleError,
    PostSaleFacts, PostSaleLifecycleUseCase, PostSaleOperation, PostSaleRepository,
    PostSaleTransaction, PostSaleTransactionFactory, PostSaleUseCase, ReturnResult,
    SaleLifecycleStatus,
};
pub use repository::{ConfirmSaleError, ConfirmSaleRepository, Reservation, SaleRepository};
