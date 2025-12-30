//! English contractions expansion
//!
//! This module provides English contractions expansion functionality,
//! equivalent to Python's `contractions` package.
//!
//! The contraction rules are embedded at compile time from JSON files
//! copied from the Python `contractions` package.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

/// Contractions data embedded at compile time
///
/// These JSON files are copied from Python `contractions` package:
/// - contractions_dict.json: Standard contractions (~180 rules)
/// - leftovers_dict.json: Leftover suffixes (~17 rules)
/// - slang_dict.json: Slang contractions (~50 rules)
const CONTRACTIONS_JSON: &str = include_str!("../data/contractions_dict.json");
const LEFTOVERS_JSON: &str = include_str!("../data/leftovers_dict.json");
const SLANG_JSON: &str = include_str!("../data/slang_dict.json");

/// Month abbreviations (added dynamically like Python version)
const MONTH_ABBREVS: &[(&str, &str)] = &[
    ("jan.", "january"),
    ("feb.", "february"),
    ("mar.", "march"),
    ("apr.", "april"),
    ("jun.", "june"),
    ("jul.", "july"),
    ("aug.", "august"),
    ("sep.", "september"),
    ("oct.", "october"),
    ("nov.", "november"),
    ("dec.", "december"),
];

/// Parsed and merged contractions mapping
///
/// Combines all three JSON sources plus month abbreviations.
/// Also handles apostrophe variants (' vs ').
static CONTRACTIONS: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Parse JSON files
    let parse_json = |json: &str| -> HashMap<String, String> {
        serde_json::from_str::<HashMap<String, String>>(json).unwrap_or_default()
    };

    // Load standard contractions
    for (k, v) in parse_json(CONTRACTIONS_JSON) {
        map.insert(k.to_lowercase(), v);
    }

    // Load leftovers
    for (k, v) in parse_json(LEFTOVERS_JSON) {
        if !v.is_empty() {
            // Skip empty mappings like "'all" → ""
            map.insert(k.to_lowercase(), v);
        }
    }

    // Load slang (optional, can be disabled)
    for (k, v) in parse_json(SLANG_JSON) {
        map.insert(k.to_lowercase(), v);
    }

    // Add month abbreviations
    for (abbrev, full) in MONTH_ABBREVS {
        map.insert(abbrev.to_string(), full.to_string());
    }

    // Handle apostrophe variants: ' (U+0027) vs ' (U+2019 curly apostrophe)
    // Add curly apostrophe variants for keys that contain straight apostrophe
    let variants: Vec<(String, String)> = map
        .iter()
        .filter(|(k, _)| k.contains('\''))
        .map(|(k, v)| (k.replace('\'', "\u{2019}"), v.clone()))
        .collect();

    for (k, v) in variants {
        map.entry(k).or_insert(v);
    }

    map
});

/// Compiled regex patterns for efficient replacement
static PATTERNS: Lazy<Vec<(Regex, String)>> = Lazy::new(|| {
    CONTRACTIONS
        .iter()
        .filter_map(|(contraction, expansion)| {
            // Build case-insensitive pattern
            // For patterns ending with '.', don't require trailing word boundary
            // For other patterns, use word boundaries
            let escaped = regex::escape(contraction);
            let pattern = if contraction.ends_with('.') {
                // Month abbreviations: match at word start, allow trailing boundary or space
                format!(r"(?i)\b{}", escaped)
            } else {
                // Standard contractions: use word boundaries
                format!(r"(?i)\b{}\b", escaped)
            };
            Regex::new(&pattern).ok().map(|re| (re, expansion.clone()))
        })
        .collect()
});

/// Expand English contractions in text
///
/// This function is equivalent to Python's `contractions.fix(text)`.
/// It handles standard contractions, leftovers, and slang.
///
/// # Arguments
/// * `text` - Input text with potential contractions
///
/// # Returns
/// Text with contractions expanded
///
/// # Example
/// ```rust,ignore
/// use wetext_rs::contractions::fix_contractions;
///
/// assert_eq!(fix_contractions("I don't know"), "I do not know");
/// assert_eq!(fix_contractions("It's gonna be fine"), "It is going to be fine");
/// assert_eq!(fix_contractions("Jan. 15th"), "january 15th");
/// ```
pub fn fix_contractions(text: &str) -> String {
    // Quick check: if no apostrophe-like chars and no known patterns, skip
    // Check for both straight apostrophe (') and curly apostrophe (')
    if !text.contains('\'') && !text.contains('\u{2019}') && !needs_expansion(text) {
        return text.to_string();
    }

    let mut result = text.to_string();

    for (pattern, expansion) in PATTERNS.iter() {
        result = pattern.replace_all(&result, expansion.as_str()).to_string();
    }

    result
}

/// Quick check for common patterns that need expansion (optimization)
fn needs_expansion(text: &str) -> bool {
    let lower = text.to_lowercase();
    // Check for common slang that doesn't contain apostrophes
    lower.contains("gonna")
        || lower.contains("wanna")
        || lower.contains("gotta")
        || lower.contains("dunno")
        || lower.contains("gimme")
        || lower.contains("lemme")
        // Check for month abbreviations
        || lower.contains("jan.")
        || lower.contains("feb.")
        || lower.contains("mar.")
        || lower.contains("apr.")
        || lower.contains("jun.")
        || lower.contains("jul.")
        || lower.contains("aug.")
        || lower.contains("sep.")
        || lower.contains("oct.")
        || lower.contains("nov.")
        || lower.contains("dec.")
}

/// Expand contractions with configuration options
///
/// # Arguments
/// * `text` - Input text
/// * `_include_slang` - Whether to expand slang (default: true in Python)
///
/// Note: For simplicity, this implementation always includes slang.
/// If you need the option to exclude slang, rebuild PATTERNS without slang entries.
#[allow(dead_code)]
pub fn fix_contractions_with_options(text: &str, _include_slang: bool) -> String {
    // Current implementation always includes slang for simplicity
    // To support this option properly, would need separate pattern sets
    fix_contractions(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_contractions() {
        assert_eq!(fix_contractions("I don't know"), "I do not know");
        // Note: regex replacement outputs lowercase expansion
        assert_eq!(fix_contractions("It's fine"), "it is fine");
        assert_eq!(fix_contractions("we're here"), "we are here");
    }

    #[test]
    fn test_slang() {
        // "I'm" is mapped to "I am" (preserves case in JSON)
        assert_eq!(fix_contractions("I'm gonna go"), "I am going to go");
        assert_eq!(fix_contractions("I wanna eat"), "I want to eat");
        assert_eq!(fix_contractions("I gotta leave"), "I got to leave");
    }

    #[test]
    fn test_month_abbreviations() {
        assert_eq!(fix_contractions("jan. 15"), "january 15");
        assert_eq!(fix_contractions("dec. 25"), "december 25");
    }

    #[test]
    fn test_curly_apostrophe() {
        // Test both straight and curly apostrophes
        assert_eq!(fix_contractions("don't"), "do not");
        assert_eq!(fix_contractions("don't"), "do not"); // curly apostrophe
    }

    #[test]
    fn test_no_contractions() {
        assert_eq!(fix_contractions("Hello world"), "Hello world");
        assert_eq!(
            fix_contractions("No contractions here"),
            "No contractions here"
        );
    }

    #[test]
    fn test_case_insensitive() {
        // Regex case-insensitive matching replaces with lowercase expansion
        assert_eq!(fix_contractions("DON'T SHOUT"), "do not SHOUT");
        assert_eq!(fix_contractions("It's OK"), "it is OK");
    }
}
