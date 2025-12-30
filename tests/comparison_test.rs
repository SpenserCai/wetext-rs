//! Comparison tests with Python wetext reference outputs

use serde::Deserialize;
use std::fs;
use wetext_rs::*;

const FST_DIR: &str = "fsts";

#[derive(Deserialize)]
struct TestCase {
    input: String,
    lang: String,
    operator: String,
    expected_output: String,
    #[serde(default)]
    error: Option<String>,
}

#[test]
#[ignore = "Run after generating reference_outputs.json with Python"]
fn test_compare_with_python() {
    let data = match fs::read_to_string("tests/reference_outputs.json") {
        Ok(data) => data,
        Err(e) => {
            println!("Skipping comparison test: {}", e);
            println!("Run 'python tests/generate_reference.py' to generate reference data");
            return;
        }
    };

    let test_cases: Vec<TestCase> =
        serde_json::from_str(&data).expect("Failed to parse JSON");

    let mut passed = 0;
    let mut failed = 0;

    for case in test_cases {
        // Skip cases that had errors in Python
        if case.error.is_some() {
            println!("Skipping '{}' (Python error)", case.input);
            continue;
        }

        let lang = match case.lang.as_str() {
            "zh" => Language::Zh,
            "en" => Language::En,
            "ja" => Language::Ja,
            _ => Language::Auto,
        };

        let operator = match case.operator.as_str() {
            "tn" => Operator::Tn,
            "itn" => Operator::Itn,
            _ => Operator::Tn,
        };

        let config = NormalizerConfig::new()
            .with_lang(lang)
            .with_operator(operator);

        let mut normalizer = Normalizer::new(FST_DIR, config);
        let result = normalizer.normalize(&case.input).unwrap();

        if result == case.expected_output {
            passed += 1;
            println!(
                "✓ PASS: '{}' ({}/{}) => '{}'",
                case.input, case.lang, case.operator, result
            );
        } else {
            failed += 1;
            println!(
                "✗ FAIL: '{}' ({}/{})\n  Expected: '{}'\n  Got:      '{}'",
                case.input, case.lang, case.operator, case.expected_output, result
            );
        }
    }

    println!("\nResults: {} passed, {} failed", passed, failed);
    assert_eq!(failed, 0, "Some comparison tests failed");
}

