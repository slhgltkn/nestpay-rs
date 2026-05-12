//! # nestpay
//!
//! NestPay / Payten Sanal POS için **Hash Version 3** protokolünü uygulayan Rust kütüphanesi.
//!
//! Halkbank ve NestPay altyapısını kullanan diğer bankalarla entegrasyon için
//! 3D, 3D_PAY ve 3D_PAY_HOSTING modellerini destekler.
//!
//! ## Hızlı Başlangıç — 3D modeli
//!
//! ```rust
//! use nestpay::{ThreeDRequest, NestPayRequest, TransactionType, Currency, CardType};
//!
//! let req = ThreeDRequest {
//!     client_id:    "100200127".into(),
//!     amount:       "95.93".into(),
//!     ok_url:       "https://example.com/ok".into(),
//!     fail_url:     "https://example.com/fail".into(),
//!     callback_url: "https://example.com/callback".into(),
//!     tran_type:    TransactionType::Auth,
//!     instalment:   String::new(),
//!     currency:     Currency::Try,
//!     rnd:          "1234567890".into(),
//!     lang:         "tr".into(),
//!     pan:          "4111111111111111".into(),
//!     cv2:          "000".into(),
//!     exp_year:     "26".into(),
//!     exp_month:    "12".into(),
//!     card_type:    CardType::Visa,
//!     bill_to_name:    Some("Ad Soyad".into()),
//!     bill_to_company: None,
//! };
//!
//! // Tüm form parametrelerini (hash dahil) al
//! let params = req.form_params("YOUR_STORE_KEY");
//!
//! // `params` artık NestPay 3D-gate URL'sine POST edilebilir
//! for (key, value) in &params {
//!     println!("{key} = {value}");
//! }
//! ```
//!
//! ## Düşük seviye API — ham parametrelerle hash hesaplama
//!
//! ```rust
//! use nestpay::hash::compute_hash;
//!
//! let params = vec![
//!     ("amount",   "95.93"),
//!     ("clientid", "100200127"),
//!     ("rnd",      "1234567890"),
//! ];
//! let hash = compute_hash(&params, "TEST1234");
//! println!("Hash: {hash}");
//! ```

pub mod hash;
pub mod models;

// Convenience re-exports so users don't need to write `nestpay::models::...`
pub use hash::{compute_hash, escape_value, verify_hash};
pub use models::{
    CardType, Currency, NestPayRequest, ThreeDPayHostingRequest, ThreeDPayRequest, ThreeDRequest,
    TransactionType,
};
