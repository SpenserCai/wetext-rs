//! Configuration types for WeText-RS

/// Text normalization operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Operator {
    /// Text Normalization: numbers → words (e.g., "123" → "一百二十三")
    #[default]
    Tn,
    /// Inverse Text Normalization: words → numbers (e.g., "一百二十三" → "123")
    Itn,
}

/// Language type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    /// Auto-detect language
    #[default]
    Auto,
    /// English
    En,
    /// Chinese
    Zh,
    /// Japanese
    Ja,
}

/// Normalizer configuration
#[derive(Debug, Clone, Default)]
pub struct NormalizerConfig {
    /// Language setting, Auto means auto-detect
    pub lang: Language,

    /// Operation type: TN or ITN
    pub operator: Operator,

    /// Whether to fix English contractions (e.g., "don't" → "do not")
    pub fix_contractions: bool,

    /// Whether to convert Traditional Chinese to Simplified Chinese
    pub traditional_to_simple: bool,

    /// Whether to convert full-width characters to half-width
    pub full_to_half: bool,

    /// Whether to remove interjections (e.g., "嗯", "啊")
    pub remove_interjections: bool,

    /// Whether to remove punctuation marks
    pub remove_puncts: bool,

    /// Whether to tag OOV (out-of-vocabulary) words
    pub tag_oov: bool,

    /// Whether to enable 0-9 digit conversion in ITN
    pub enable_0_to_9: bool,

    /// Whether to remove erhua (儿化音) (e.g., "哪儿" → "哪")
    pub remove_erhua: bool,
}

impl NormalizerConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the language
    pub fn with_lang(mut self, lang: Language) -> Self {
        self.lang = lang;
        self
    }

    /// Set the operator
    pub fn with_operator(mut self, operator: Operator) -> Self {
        self.operator = operator;
        self
    }

    /// Set whether to fix contractions
    pub fn with_fix_contractions(mut self, fix: bool) -> Self {
        self.fix_contractions = fix;
        self
    }

    /// Set whether to convert traditional to simplified Chinese
    pub fn with_traditional_to_simple(mut self, convert: bool) -> Self {
        self.traditional_to_simple = convert;
        self
    }

    /// Set whether to convert full-width to half-width
    pub fn with_full_to_half(mut self, convert: bool) -> Self {
        self.full_to_half = convert;
        self
    }

    /// Set whether to remove interjections
    pub fn with_remove_interjections(mut self, remove: bool) -> Self {
        self.remove_interjections = remove;
        self
    }

    /// Set whether to remove punctuation
    pub fn with_remove_puncts(mut self, remove: bool) -> Self {
        self.remove_puncts = remove;
        self
    }

    /// Set whether to remove erhua
    pub fn with_remove_erhua(mut self, remove: bool) -> Self {
        self.remove_erhua = remove;
        self
    }

    /// Set whether to tag OOV words
    pub fn with_tag_oov(mut self, tag: bool) -> Self {
        self.tag_oov = tag;
        self
    }

    /// Set whether to enable 0-9 conversion in ITN
    pub fn with_enable_0_to_9(mut self, enable: bool) -> Self {
        self.enable_0_to_9 = enable;
        self
    }
}
