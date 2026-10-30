//! Unicode-safe password text preparation.
//!
//! `passtext-core` validates explicit processing limits, optionally applies NFC
//! normalization, records content-free metadata, and converts safely between
//! Unicode scalar-value indices and UTF-8 byte offsets. It does not score,
//! hash, authenticate, or otherwise judge passwords.
//!
//! Password text is deliberately available only through accessors on
//! [`PreparedPassword`]. Its [`Debug`](std::fmt::Debug) implementation and all
//! error messages omit the text.
//!
//! # Example
//!
//! ```
//! use passtext_core::{process_password, NormalizationMode, PassTextConfig};
//!
//! let config = PassTextConfig::default().with_normalization(NormalizationMode::Nfc);
//! let prepared = process_password("e\u{301}", config)?;
//!
//! assert_eq!(prepared.original_text(), "e\u{301}");
//! assert_eq!(prepared.processed_text(), "é");
//! assert!(prepared.metadata().normalization_changed());
//! # Ok::<(), passtext_core::PassTextError>(())
//! ```

#![forbid(unsafe_code)]

mod config;
mod error;
mod indexing;
mod metadata;
mod normalization;
mod representation;
mod unicode;
mod validation;

pub use config::{NormalizationMode, PassTextConfig};
pub use error::{ConfigField, ErrorCategory, LimitUnit, PassTextError, ProcessingStage};
pub use indexing::{
    codepoint_index_for_utf8_offset, slice_text, utf8_offset_for_codepoint, IndexError, TextSpan,
};
pub use metadata::TextMetadata;
pub use normalization::UNICODE_NORMALIZATION_VERSION;
pub use representation::PreparedPassword;
pub use validation::{
    process_codepoints, process_owned_password, process_password, process_password_with_default,
    process_utf8,
};
