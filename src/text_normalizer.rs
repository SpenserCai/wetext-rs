//! FST-based text normalizer
//!
//! This module provides FST (Finite State Transducer) based text normalization,
//! equivalent to kaldifst.TextNormalizer in Python.

use std::path::Path;

use rustfst::algorithms::compose::compose;
use rustfst::algorithms::shortest_path;
use rustfst::fst_impls::VectorFst;
use rustfst::fst_traits::SerializableFst;
use rustfst::prelude::*;
use rustfst::semirings::TropicalWeight;
use rustfst::utils::{acceptor, decode_linear_fst};
use rustfst::{Label, EPS_LABEL};

use crate::error::{Result, WeTextError};

/// FST-based text normalizer
///
/// Equivalent to kaldifst.TextNormalizer in Python
pub struct FstTextNormalizer {
    fst: VectorFst<TropicalWeight>,
}

impl FstTextNormalizer {
    /// Load FST from file
    ///
    /// # Arguments
    /// * `path` - Path to the FST file (OpenFST binary format)
    ///
    /// # Returns
    /// A new FstTextNormalizer instance
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(WeTextError::FstNotFound(path.display().to_string()));
        }

        // VectorFst::read() loads OpenFST binary format
        // This method comes from the SerializableFst trait
        let fst = VectorFst::<TropicalWeight>::read(path)
            .map_err(|e| WeTextError::FstLoadError(e.to_string()))?;

        Ok(Self { fst })
    }

    /// Apply FST for text transformation
    ///
    /// Implementation flow:
    /// 1. Convert input string to linear FST (acceptor) using UTF-8 bytes
    /// 2. Compose with the loaded FST
    /// 3. Find shortest path
    /// 4. Extract output string from the path
    ///
    /// # Arguments
    /// * `input` - Input text to normalize
    ///
    /// # Returns
    /// Normalized text string
    pub fn normalize(&self, input: &str) -> Result<String> {
        if input.is_empty() {
            return Ok(String::new());
        }

        // Step 1: Convert input string to linear FST using UTF-8 bytes
        // WeText FSTs use UTF-8 byte encoding for labels
        let labels: Vec<Label> = input.as_bytes().iter().map(|&b| b as Label).collect();
        let input_fst: VectorFst<TropicalWeight> = acceptor(&labels, TropicalWeight::one());

        // Step 2: Compose with the normalizer FST
        // Note: compose() requires output type to implement AllocableFst
        // Explicitly specify all type parameters for compose
        let composed: VectorFst<TropicalWeight> = compose::<
            TropicalWeight,
            VectorFst<TropicalWeight>,
            VectorFst<TropicalWeight>,
            VectorFst<TropicalWeight>,
            _,
            _,
        >(&input_fst, &self.fst)
        .map_err(|e| WeTextError::FstOperationError(format!("compose failed: {}", e)))?;

        // Check if compose result is empty (no match)
        if composed.num_states() == 0 {
            // If no match, return original input (same as kaldifst behavior)
            return Ok(input.to_string());
        }

        // Step 3: Find shortest path
        let best_path: VectorFst<TropicalWeight> = shortest_path(&composed)
            .map_err(|e| WeTextError::FstOperationError(format!("shortest_path failed: {}", e)))?;

        // Check if shortest_path result is empty
        if best_path.num_states() == 0 {
            return Ok(input.to_string());
        }

        // Step 4: Extract output string using decode_linear_fst
        self.fst_to_string(&best_path)
    }

    /// Extract output string from linear FST
    fn fst_to_string(&self, fst: &VectorFst<TropicalWeight>) -> Result<String> {
        if fst.num_states() == 0 {
            return Ok(String::new());
        }

        // Use decode_linear_fst to get the path
        let path =
            decode_linear_fst(fst).map_err(|e| WeTextError::FstOperationError(e.to_string()))?;

        // FST labels can be either:
        // 1. Unicode code points (for CJK characters, code > 255)
        // 2. UTF-8 bytes (for ASCII, code < 256)
        // We need to handle both cases

        // Check if labels look like UTF-8 bytes (all < 256) or Unicode code points
        let has_high_codepoint = path
            .olabels
            .iter()
            .any(|&label| label != EPS_LABEL && label > 255);

        if has_high_codepoint {
            // Labels are Unicode code points - convert directly
            let output: String = path
                .olabels
                .iter()
                .filter_map(|&label| {
                    if label == EPS_LABEL {
                        None
                    } else {
                        char::from_u32(label)
                    }
                })
                .collect();
            Ok(output)
        } else {
            // Labels are likely UTF-8 bytes - collect and decode
            let bytes: Vec<u8> = path
                .olabels
                .iter()
                .filter_map(|&label| {
                    if label == EPS_LABEL {
                        None
                    } else {
                        Some(label as u8)
                    }
                })
                .collect();

            String::from_utf8(bytes).map_err(|e| {
                WeTextError::FstOperationError(format!("Invalid UTF-8 in FST output: {}", e))
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acceptor_creation() {
        let labels: Vec<Label> = "hello".chars().map(|c| c as Label).collect();
        let fst: VectorFst<TropicalWeight> = acceptor(&labels, TropicalWeight::one());
        assert_eq!(fst.num_states(), 6); // 5 chars + 1 (start state)
    }
}
