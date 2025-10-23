pub mod stellar;
pub mod stellar_service;
pub mod notifications;
pub mod contract_client;
pub mod payment_service;

pub use self::stellar::StellarService;
pub use self::stellar_service::{StellarService as NewStellarService, WalletInfo, BalanceInfo, TransactionInfo};