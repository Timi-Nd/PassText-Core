//! Typed, content-free processing errors.

use std::error::Error;
use std::fmt;

/// Broad error categories corresponding to the Python reference exceptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// An external representation was not valid Unicode scalar text.
    InvalidUnicode,
    /// A configured processing limit was exceeded.
    ProcessingLimit,
    /// A configuration value was invalid.
    InvalidConfiguration,
    /// Valid input could not be processed as promised.
    ProcessingFailure,
}

/// Whether a limit applies to the source or normalized text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcessingStage {
    /// Text before normalization.
    Original,
    /// Text after normalization.
    Processed,
}

/// The unit measured by a processing limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LimitUnit {
    /// Unicode scalar values (called code points by the public contract).
    UnicodeCodepoints,
    /// Bytes in the strict UTF-8 representation.
    Utf8Bytes,
}

/// A configuration field that failed validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigField {
    /// The maximum UTF-8 byte count was zero.
    MaxUtf8Bytes,
    /// The maximum Unicode code-point count was zero.
    MaxCodepoints,
    /// A string did not name a supported normalization mode.
    Normalization,
}

/// An error produced while configuring or processing password text.
///
/// Variants intentionally contain no password data. They may safely be
/// formatted for diagnostics, although callers should still avoid logging
/// sensitive surrounding application state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PassTextError {
    /// Bytes or integer code points could not form valid Unicode scalar text.
    InvalidUnicode,
    /// An original or processed text limit was exceeded.
    ProcessingLimit {
        /// The processing stage at which the limit was checked.
        stage: ProcessingStage,
        /// The unit whose maximum was exceeded.
        unit: LimitUnit,
    },
    /// A configuration value was invalid.
    InvalidConfiguration {
        /// The invalid field.
        field: ConfigField,
    },
    /// Processing valid text failed unexpectedly.
    ProcessingFailure,
}

impl PassTextError {
    /// Return the stable high-level category for this error.
    #[must_use]
    pub const fn category(self) -> ErrorCategory {
        match self {
            Self::InvalidUnicode => ErrorCategory::InvalidUnicode,
            Self::ProcessingLimit { .. } => ErrorCategory::ProcessingLimit,
            Self::InvalidConfiguration { .. } => ErrorCategory::InvalidConfiguration,
            Self::ProcessingFailure => ErrorCategory::ProcessingFailure,
        }
    }
}

impl fmt::Display for PassTextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUnicode => formatter.write_str("input contains unsupported Unicode"),
            Self::ProcessingLimit {
                stage: ProcessingStage::Original,
                unit: LimitUnit::UnicodeCodepoints,
            } => {
                formatter.write_str("password exceeds configured maximum Unicode code-point count")
            }
            Self::ProcessingLimit {
                stage: ProcessingStage::Original,
                unit: LimitUnit::Utf8Bytes,
            } => formatter.write_str("password exceeds configured maximum UTF-8 byte count"),
            Self::ProcessingLimit {
                stage: ProcessingStage::Processed,
                unit: LimitUnit::UnicodeCodepoints,
            } => formatter.write_str(
                "processed password exceeds configured maximum Unicode code-point count",
            ),
            Self::ProcessingLimit {
                stage: ProcessingStage::Processed,
                unit: LimitUnit::Utf8Bytes,
            } => formatter
                .write_str("processed password exceeds configured maximum UTF-8 byte count"),
            Self::InvalidConfiguration {
                field: ConfigField::MaxUtf8Bytes,
            } => formatter.write_str("max_utf8_bytes must be greater than zero"),
            Self::InvalidConfiguration {
                field: ConfigField::MaxCodepoints,
            } => formatter.write_str("max_codepoints must be greater than zero"),
            Self::InvalidConfiguration {
                field: ConfigField::Normalization,
            } => formatter.write_str("normalization must be 'none' or 'nfc'"),
            Self::ProcessingFailure => formatter.write_str("password processing failed"),
        }
    }
}

impl Error for PassTextError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_have_stable_categories_and_no_payload() {
        let error = PassTextError::ProcessingLimit {
            stage: ProcessingStage::Original,
            unit: LimitUnit::Utf8Bytes,
        };
        assert_eq!(error.category(), ErrorCategory::ProcessingLimit);
        assert_eq!(
            error.to_string(),
            "password exceeds configured maximum UTF-8 byte count"
        );
    }
}
