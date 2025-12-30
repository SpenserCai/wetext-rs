#!/usr/bin/env python3
"""Generate Python wetext reference outputs for Rust comparison tests"""

import json
import sys

try:
    from wetext import Normalizer
except ImportError:
    print("Error: wetext not installed. Please install it first.")
    print("Try: pip install wetext")
    sys.exit(1)

test_cases = [
    # Chinese TN (Text Normalization)
    {"text": "123", "lang": "zh", "operator": "tn"},
    {"text": "2024年1月15日", "lang": "zh", "operator": "tn"},
    {"text": "100元", "lang": "zh", "operator": "tn"},
    {"text": "下午3点30分", "lang": "zh", "operator": "tn"},
    {"text": "3/4", "lang": "zh", "operator": "tn"},
    {"text": "1.5", "lang": "zh", "operator": "tn"},
    
    # Chinese ITN (Inverse Text Normalization)
    {"text": "一百二十三", "lang": "zh", "operator": "itn"},
    {"text": "二零二四年一月十五日", "lang": "zh", "operator": "itn"},
    {"text": "四分之三", "lang": "zh", "operator": "itn"},
    
    # English TN
    {"text": "$100", "lang": "en", "operator": "tn"},
    {"text": "January 15, 2024", "lang": "en", "operator": "tn"},
    {"text": "3.14", "lang": "en", "operator": "tn"},
    
    # Japanese TN
    {"text": "100円", "lang": "ja", "operator": "tn"},
    {"text": "2024年", "lang": "ja", "operator": "tn"},
    {"text": "3月15日", "lang": "ja", "operator": "tn"},
    
    # Japanese ITN
    {"text": "百円", "lang": "ja", "operator": "itn"},
    
    # Edge cases
    {"text": "", "lang": "zh", "operator": "tn"},
    {"text": "没有数字", "lang": "zh", "operator": "tn"},
    {"text": "Hello World", "lang": "en", "operator": "tn"},
    {"text": "   123   ", "lang": "zh", "operator": "tn"},  # Leading/trailing whitespace
]

results = []
for case in test_cases:
    try:
        normalizer = Normalizer(lang=case["lang"], operator=case["operator"])
        output = normalizer.normalize(case["text"])
        results.append({
            "input": case["text"],
            "lang": case["lang"],
            "operator": case["operator"],
            "expected_output": output,
        })
    except Exception as e:
        print(f"Warning: Failed to process '{case['text']}' ({case['lang']}/{case['operator']}): {e}")
        results.append({
            "input": case["text"],
            "lang": case["lang"],
            "operator": case["operator"],
            "expected_output": case["text"],  # Fallback to original
            "error": str(e),
        })

with open("tests/reference_outputs.json", "w", encoding="utf-8") as f:
    json.dump(results, f, ensure_ascii=False, indent=2)

print(f"Generated {len(results)} test cases to tests/reference_outputs.json")

