//! Main Normalizer implementation
//!
//! This module provides the main Normalizer struct that orchestrates
//! the text normalization pipeline.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config::{Language, NormalizerConfig, Operator};
use crate::contractions::fix_contractions;
use crate::error::{Result, WeTextError};
use crate::text_normalizer::FstTextNormalizer;
use crate::token_parser::TokenParser;

/// FST file cache for lazy loading
struct FstCache {
    fsts: HashMap<String, FstTextNormalizer>,
    fst_dir: PathBuf,
}

impl FstCache {
    fn new<P: AsRef<Path>>(fst_dir: P) -> Self {
        Self {
            fsts: HashMap::new(),
            fst_dir: fst_dir.as_ref().to_path_buf(),
        }
    }

    fn get_or_load(&mut self, relative_path: &str) -> Result<&FstTextNormalizer> {
        if !self.fsts.contains_key(relative_path) {
            let full_path = self.fst_dir.join(relative_path);
            let normalizer = FstTextNormalizer::from_file(&full_path)?;
            self.fsts.insert(relative_path.to_string(), normalizer);
        }
        Ok(self.fsts.get(relative_path).unwrap())
    }
}

/// WeText Normalizer
///
/// Main entry point for text normalization functionality.
/// Supports Text Normalization (TN) and Inverse Text Normalization (ITN)
/// for Chinese, English, and Japanese.
///
/// # Example
/// ```rust,ignore
/// use wetext_rs::{Normalizer, NormalizerConfig, Language};
///
/// let config = NormalizerConfig::new().with_lang(Language::Zh);
/// let mut normalizer = Normalizer::new("path/to/fsts", config);
/// let result = normalizer.normalize("2024年").unwrap();
/// // Result: "二零二四年"
/// ```
pub struct Normalizer {
    config: NormalizerConfig,
    cache: FstCache,
}

impl Normalizer {
    /// Create a new Normalizer
    ///
    /// # Arguments
    /// * `fst_dir` - Directory containing FST weight files
    /// * `config` - Normalizer configuration
    pub fn new<P: AsRef<Path>>(fst_dir: P, config: NormalizerConfig) -> Self {
        Self {
            config,
            cache: FstCache::new(fst_dir),
        }
    }

    /// Create a Normalizer with default configuration
    pub fn with_defaults<P: AsRef<Path>>(fst_dir: P) -> Self {
        Self::new(fst_dir, NormalizerConfig::default())
    }

    /// Normalize text using the configured settings
    pub fn normalize(&mut self, text: &str) -> Result<String> {
        self.normalize_with_config(text, &self.config.clone())
    }

    /// Normalize text with a specific configuration
    pub fn normalize_with_config(
        &mut self,
        text: &str,
        config: &NormalizerConfig,
    ) -> Result<String> {
        let mut text = text.to_string();

        // 1. Fix English contractions
        if config.fix_contractions && text.contains('\'') {
            text = fix_contractions(&text);
        }

        // 2. Preprocessing
        text = self.preprocess(&text, config)?;

        // 3. Detect language
        let lang = if config.lang == Language::Auto {
            Self::detect_language(&text)
        } else {
            config.lang
        };

        // 4. Check if normalization is needed
        if self.should_normalize(&text, config.operator, config.remove_erhua) {
            // English ITN is not supported in Python wetext (raises NotImplementedError).
            // Fallback to Chinese ITN as a workaround, matching Python behavior.
            let lang = if lang == Language::En && config.operator == Operator::Itn {
                Language::Zh
            } else {
                lang
            };

            // 4.1 Tagger: tag entities
            text = self.tag(&text, lang, config)?;

            // 4.2 Reorder: reorder token fields
            text = self.reorder(&text, lang, config.operator)?;

            // 4.3 Verbalizer: convert to spoken form
            text = self.verbalize(&text, lang, config)?;
        }

        // 5. Postprocessing
        text = self.postprocess(&text, config)?;

        Ok(text)
    }

    /// Detect text language
    ///
    /// **Note:** This implementation extends the original Python version with Japanese detection.
    /// Python wetext only detects Chinese vs English. This Rust version adds Japanese support
    /// by detecting Hiragana/Katakana characters.
    ///
    /// Detection priority:
    /// 1. Japanese (Hiragana/Katakana) - Rust extension, not in Python version
    /// 2. Chinese (CJK Unified Ideographs)
    /// 3. Numeric-only text (digits, punctuation, symbols) - treated as Chinese
    /// 4. Default to English
    fn detect_language(text: &str) -> Language {
        let mut has_cjk = false;
        let mut has_alpha = false;

        for ch in text.chars() {
            // [Rust Extension] Japanese detection via Hiragana/Katakana
            // Japanese Hiragana: U+3040 - U+309F
            // Japanese Katakana: U+30A0 - U+30FF
            // Note: Python wetext does NOT have this detection - it would return "zh" for Japanese text
            if ('\u{3040}'..='\u{309f}').contains(&ch) || ('\u{30a0}'..='\u{30ff}').contains(&ch) {
                return Language::Ja;
            }

            // CJK Unified Ideographs: U+4E00 - U+9FFF
            // Note: These are shared between Chinese and Japanese
            // If we find hiragana/katakana, it's Japanese; otherwise treat as Chinese
            if ('\u{4e00}'..='\u{9fff}').contains(&ch) {
                has_cjk = true;
            }

            // Track if there are any ASCII alphabetic characters
            if ch.is_ascii_alphabetic() {
                has_alpha = true;
            }
        }

        // If contains CJK but no Japanese-specific characters, treat as Chinese
        if has_cjk {
            return Language::Zh;
        }

        // Numeric-only text (no alphabetic characters) treated as Chinese
        // This covers cases like "123", "3/4", "1.5", "2024年" (when year char is not present)
        if !text.is_empty() && !has_alpha {
            return Language::Zh;
        }

        Language::En
    }

    /// Check if normalization is needed
    fn should_normalize(&self, text: &str, operator: Operator, remove_erhua: bool) -> bool {
        if operator == Operator::Tn {
            // TN: needs normalization if contains digits
            if text.chars().any(|c| c.is_ascii_digit()) {
                return true;
            }
            // Or if need to remove erhua
            if remove_erhua && (text.contains('儿') || text.contains('兒')) {
                return true;
            }
            false
        } else {
            // ITN: non-empty text needs processing
            !text.is_empty()
        }
    }

    /// Preprocessing step
    fn preprocess(&mut self, text: &str, config: &NormalizerConfig) -> Result<String> {
        let mut result = text.trim().to_string();

        if config.traditional_to_simple {
            let fst = self.cache.get_or_load("traditional_to_simple.fst")?;
            result = fst.normalize(&result)?;
        }

        Ok(result)
    }

    /// Postprocessing step
    fn postprocess(&mut self, text: &str, config: &NormalizerConfig) -> Result<String> {
        let mut result = text.to_string();

        if config.full_to_half {
            let fst = self.cache.get_or_load("full_to_half.fst")?;
            result = fst.normalize(&result)?;
        }

        if config.remove_interjections {
            let fst = self.cache.get_or_load("remove_interjections.fst")?;
            result = fst.normalize(&result)?;
        }

        if config.remove_puncts {
            let fst = self.cache.get_or_load("remove_puncts.fst")?;
            result = fst.normalize(&result)?;
        }

        if config.tag_oov {
            let fst = self.cache.get_or_load("tag_oov.fst")?;
            result = fst.normalize(&result)?;
        }

        Ok(result.trim().to_string())
    }

    /// Tag entities using tagger FST
    fn tag(&mut self, text: &str, lang: Language, config: &NormalizerConfig) -> Result<String> {
        let fst_path = match (lang, config.operator) {
            (Language::En, Operator::Tn) => "en/tn/tagger.fst",
            (Language::Zh, Operator::Tn) => "zh/tn/tagger.fst",
            (Language::Zh, Operator::Itn) => {
                if config.enable_0_to_9 {
                    "zh/itn/tagger_enable_0_to_9.fst"
                } else {
                    "zh/itn/tagger.fst"
                }
            }
            (Language::Ja, Operator::Tn) => "ja/tn/tagger.fst",
            (Language::Ja, Operator::Itn) => {
                if config.enable_0_to_9 {
                    "ja/itn/tagger_enable_0_to_9.fst"
                } else {
                    "ja/itn/tagger.fst"
                }
            }
            _ => return Err(WeTextError::InvalidLanguage(format!("{:?}", lang))),
        };

        let fst = self.cache.get_or_load(fst_path)?;
        let result = fst.normalize(text)?;
        Ok(result.trim().to_string())
    }

    /// Reorder token fields
    fn reorder(&self, text: &str, lang: Language, operator: Operator) -> Result<String> {
        let parser = TokenParser::new(lang, operator);
        parser.reorder(text)
    }

    /// Verbalize using verbalizer FST
    fn verbalize(
        &mut self,
        text: &str,
        lang: Language,
        config: &NormalizerConfig,
    ) -> Result<String> {
        let fst_path = match (lang, config.operator) {
            (Language::En, Operator::Tn) => "en/tn/verbalizer.fst",
            (Language::Zh, Operator::Tn) => {
                if config.remove_erhua {
                    "zh/tn/verbalizer_remove_erhua.fst"
                } else {
                    "zh/tn/verbalizer.fst"
                }
            }
            (Language::Zh, Operator::Itn) => "zh/itn/verbalizer.fst",
            (Language::Ja, Operator::Tn) => "ja/tn/verbalizer.fst",
            (Language::Ja, Operator::Itn) => "ja/itn/verbalizer.fst",
            _ => return Err(WeTextError::InvalidLanguage(format!("{:?}", lang))),
        };

        let fst = self.cache.get_or_load(fst_path)?;
        let result = fst.normalize(text)?;
        Ok(result.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language() {
        // English
        assert_eq!(Normalizer::detect_language("hello world"), Language::En);
        assert_eq!(Normalizer::detect_language("Hello, World!"), Language::En);

        // Chinese
        assert_eq!(Normalizer::detect_language("你好世界"), Language::Zh);
        assert_eq!(Normalizer::detect_language("今天是2024年"), Language::Zh);

        // Japanese (Hiragana/Katakana triggers Japanese detection)
        assert_eq!(Normalizer::detect_language("こんにちは"), Language::Ja); // Hiragana
        assert_eq!(Normalizer::detect_language("カタカナ"), Language::Ja); // Katakana
        assert_eq!(Normalizer::detect_language("東京タワー"), Language::Ja); // Mixed Kanji + Katakana

        // Pure digits treated as Chinese (common TTS use case)
        assert_eq!(Normalizer::detect_language("123"), Language::Zh);
        assert_eq!(Normalizer::detect_language("2024"), Language::Zh);

        // Edge cases
        assert_eq!(Normalizer::detect_language(""), Language::En); // Empty defaults to English
    }
}
