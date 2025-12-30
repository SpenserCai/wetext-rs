//! # WeText-RS: Text Normalization Library
//!
//! A Rust implementation of WeText for text normalization in TTS (Text-to-Speech) applications.
//!
//! ## Features
//!
//! - **Text Normalization (TN)**: Convert numbers, dates, currency to spoken form
//! - **Inverse Text Normalization (ITN)**: Convert spoken form back to written form
//! - **Multi-language support**: Chinese (zh), English (en), Japanese (ja)
//!
//! ## Example
//!
//! ```rust,ignore
//! use wetext_rs::{Normalizer, NormalizerConfig, Language};
//!
//! let config = NormalizerConfig::new()
//!     .with_lang(Language::Zh);
//!
//! let mut normalizer = Normalizer::new("path/to/fsts", config);
//! let result = normalizer.normalize("2024年1月15日").unwrap();
//! println!("{}", result);  // 二零二四年一月十五日
//! ```

mod config;
mod contractions;
mod error;
mod normalizer;
mod text_normalizer;
mod token_parser;

pub use config::{Language, NormalizerConfig, Operator};
pub use error::{Result, WeTextError};
pub use normalizer::Normalizer;

/// Convenience function: normalize text with default configuration
///
/// # Arguments
/// * `fst_dir` - Directory containing FST weight files
/// * `text` - Text to normalize
///
/// # Returns
/// Normalized text string
///
/// # Example
/// ```rust,ignore
/// let result = wetext_rs::normalize("path/to/fsts", "123").unwrap();
/// assert_eq!(result, "一百二十三");
/// ```
pub fn normalize<P: AsRef<std::path::Path>>(fst_dir: P, text: &str) -> Result<String> {
    let mut normalizer = Normalizer::with_defaults(fst_dir);
    normalizer.normalize(text)
}
