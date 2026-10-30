//! Immutable processing configuration.

use std::fmt;
use std::str::FromStr;

use crate::{ConfigField, PassTextError};

/// Default maximum size of an input's UTF-8 representation.
pub const DEFAULT_MAX_UTF8_BYTES: usize = 4096;

/// Default maximum number of Unicode scalar values in an input.
pub const DEFAULT_MAX_CODEPOINTS: usize = 1024;

/// An explicitly selected Unicode normalization operation.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NormalizationMode {
    /// Preserve the input exactly.
    #[default]
    None,
    /// Apply Unicode Normalization Form C.
    Nfc,
}

impl NormalizationMode {
    /// Return the portable lowercase name used by the shared fixtures.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Nfc => "nfc",
        }
    }
}

impl fmt::Display for NormalizationMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for NormalizationMode {
    type Err = PassTextError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "none" => Ok(Self::None),
            "nfc" => Ok(Self::Nfc),
            _ => Err(PassTextError::InvalidConfiguration {
                field: ConfigField::Normalization,
            }),
        }
    }
}

/// Validated, immutable limits and normalization settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PassTextConfig {
    max_utf8_bytes: usize,
    max_codepoints: usize,
    normalization: NormalizationMode,
}

impl PassTextConfig {
    /// Construct a configuration, rejecting zero-valued limits.
    ///
    /// Validation follows the Python reference order: the UTF-8 byte limit is
    /// checked before the code-point limit.
    pub const fn new(
        max_utf8_bytes: usize,
        max_codepoints: usize,
        normalization: NormalizationMode,
    ) -> Result<Self, PassTextError> {
        if max_utf8_bytes == 0 {
            return Err(PassTextError::InvalidConfiguration {
                field: ConfigField::MaxUtf8Bytes,
            });
        }
        if max_codepoints == 0 {
            return Err(PassTextError::InvalidConfiguration {
                field: ConfigField::MaxCodepoints,
            });
        }
        Ok(Self {
            max_utf8_bytes,
            max_codepoints,
            normalization,
        })
    }

    /// Return the maximum permitted strict UTF-8 byte count.
    #[must_use]
    pub const fn max_utf8_bytes(self) -> usize {
        self.max_utf8_bytes
    }

    /// Return the maximum permitted Unicode scalar-value count.
    #[must_use]
    pub const fn max_codepoints(self) -> usize {
        self.max_codepoints
    }

    /// Return the configured normalization mode.
    #[must_use]
    pub const fn normalization(self) -> NormalizationMode {
        self.normalization
    }

    /// Return a copy with a different validated UTF-8 byte limit.
    pub const fn with_max_utf8_bytes(self, value: usize) -> Result<Self, PassTextError> {
        Self::new(value, self.max_codepoints, self.normalization)
    }

    /// Return a copy with a different validated code-point limit.
    pub const fn with_max_codepoints(self, value: usize) -> Result<Self, PassTextError> {
        Self::new(self.max_utf8_bytes, value, self.normalization)
    }

    /// Return a copy with a different normalization mode.
    #[must_use]
    pub const fn with_normalization(self, value: NormalizationMode) -> Self {
        Self {
            normalization: value,
            ..self
        }
    }
}

impl Default for PassTextConfig {
    fn default() -> Self {
        Self {
            max_utf8_bytes: DEFAULT_MAX_UTF8_BYTES,
            max_codepoints: DEFAULT_MAX_CODEPOINTS,
            normalization: NormalizationMode::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_reference() {
        let config = PassTextConfig::default();
        assert_eq!(config.max_utf8_bytes(), 4096);
        assert_eq!(config.max_codepoints(), 1024);
        assert_eq!(config.normalization(), NormalizationMode::None);
    }

    #[test]
    fn zero_limits_and_unknown_mode_are_rejected() {
        assert!(matches!(
            PassTextConfig::new(0, 1, NormalizationMode::None),
            Err(PassTextError::InvalidConfiguration {
                field: ConfigField::MaxUtf8Bytes
            })
        ));
        assert!(matches!(
            PassTextConfig::new(1, 0, NormalizationMode::None),
            Err(PassTextError::InvalidConfiguration {
                field: ConfigField::MaxCodepoints
            })
        ));
        assert!("NFC".parse::<NormalizationMode>().is_err());
    }
}
