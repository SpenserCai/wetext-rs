# wetext-rs

A Rust implementation of [WeText](https://github.com/pengzhendong/wetext) for text normalization in TTS (Text-to-Speech) applications.

## Background

This project is a Rust port of the Python [wetext](https://github.com/pengzhendong/wetext) library, which provides a lightweight runtime for WeTextProcessing without depending on Pynini. The primary motivation for creating this Rust implementation is to:

1. **Integrate with the Candle ecosystem** - Enable seamless integration with Rust-based ML frameworks like [Candle](https://github.com/huggingface/candle), eliminating Python dependencies in production deployments
2. **Improve performance** - Leverage Rust's memory safety and zero-cost abstractions for faster text processing
3. **Enable standalone deployment** - Create a single binary that can be deployed without Python runtime

The original Python implementation uses [kaldifst](https://github.com/k2-fsa/kaldifst) for FST operations. This Rust version uses [rustfst](https://github.com/Garvys/rustfst), a pure Rust implementation of OpenFST, to achieve the same functionality.

## Features

- **Text Normalization (TN)**: Convert numbers, dates, currency to spoken form
  - `"2024年1月15日"` → `"二零二四年一月十五日"`
  - `"$100"` → `"one hundred dollars"`
- **Inverse Text Normalization (ITN)**: Convert spoken form back to written form
  - `"一百二十三"` → `"123"`
- **Multi-language support**: Chinese (zh), English (en), Japanese (ja)
- **English contractions expansion**: `"don't"` → `"do not"`
- **Various text preprocessing options**:
  - Traditional to Simplified Chinese conversion
  - Full-width to half-width character conversion
  - Interjection removal
  - Punctuation removal
  - Erhua (儿化音) removal

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
wetext-rs = "0.1"
```

## FST Weight Files

This library requires FST (Finite State Transducer) weight files for text normalization. The weight files can be downloaded from:

**ModelScope**: [pengzhendong/wetext](https://modelscope.cn/models/pengzhendong/wetext)

Download the weight files and organize them in the following structure:

```
fsts/
├── traditional_to_simple.fst
├── full_to_half.fst
├── remove_interjections.fst
├── remove_puncts.fst
├── tag_oov.fst
├── en/
│   └── tn/
│       ├── tagger.fst
│       └── verbalizer.fst
├── zh/
│   ├── tn/
│   │   ├── tagger.fst
│   │   ├── verbalizer.fst
│   │   └── verbalizer_remove_erhua.fst
│   └── itn/
│       ├── tagger.fst
│       ├── tagger_enable_0_to_9.fst
│       └── verbalizer.fst
└── ja/
    ├── tn/
    │   ├── tagger.fst
    │   └── verbalizer.fst
    └── itn/
        ├── tagger.fst
        ├── tagger_enable_0_to_9.fst
        └── verbalizer.fst
```

You can download using ModelScope CLI:

```bash
pip install modelscope
modelscope download --model pengzhendong/wetext --local_dir ./fsts
```

Or using Git LFS:

```bash
git lfs install
git clone https://www.modelscope.cn/pengzhendong/wetext.git fsts
```

## Usage

### Basic Usage

```rust
use wetext_rs::{Normalizer, NormalizerConfig, Language, Operator};

// Create normalizer with default settings (Chinese TN, auto language detection)
let mut normalizer = Normalizer::with_defaults("path/to/fsts");

// Normalize text
let result = normalizer.normalize("2024年1月15日").unwrap();
println!("{}", result);  // 二零二四年一月十五日
```

### With Configuration

```rust
use wetext_rs::{Normalizer, NormalizerConfig, Language, Operator};

// Configure for specific language and operation
let config = NormalizerConfig::new()
    .with_lang(Language::Zh)
    .with_operator(Operator::Tn)
    .with_fix_contractions(true)
    .with_traditional_to_simple(true);

let mut normalizer = Normalizer::new("path/to/fsts", config);
let result = normalizer.normalize("100元").unwrap();
println!("{}", result);  // 一百元
```

### Inverse Text Normalization (ITN)

```rust
use wetext_rs::{Normalizer, NormalizerConfig, Language, Operator};

let config = NormalizerConfig::new()
    .with_lang(Language::Zh)
    .with_operator(Operator::Itn);

let mut normalizer = Normalizer::new("path/to/fsts", config);
let result = normalizer.normalize("一百二十三").unwrap();
println!("{}", result);  // 123
```

### Convenience Function

```rust
use wetext_rs::normalize;

let result = normalize("path/to/fsts", "123").unwrap();
println!("{}", result);  // 幺二三
```

## Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `lang` | `Auto` | Language: `Auto`, `En`, `Zh`, `Ja` |
| `operator` | `Tn` | Operation: `Tn` (text normalization), `Itn` (inverse) |
| `fix_contractions` | `false` | Expand English contractions |
| `traditional_to_simple` | `false` | Convert Traditional to Simplified Chinese |
| `full_to_half` | `false` | Convert full-width to half-width characters |
| `remove_interjections` | `false` | Remove interjections (e.g., "嗯", "啊") |
| `remove_puncts` | `false` | Remove punctuation marks |
| `tag_oov` | `false` | Tag out-of-vocabulary words |
| `enable_0_to_9` | `false` | Enable 0-9 digit conversion in ITN |
| `remove_erhua` | `false` | Remove erhua (儿化音) |

## Examples

### Chinese Text Normalization

| Input | Output |
|-------|--------|
| `123` | `幺二三` |
| `2024年` | `二零二四年` |
| `2024年1月15日` | `二零二四年一月十五日` |
| `下午3点30分` | `下午三点三十分` |
| `100元` | `一百元` |
| `3/4` | `四分之三` |
| `1.5` | `一点五` |

### Chinese Inverse Text Normalization

| Input | Output |
|-------|--------|
| `一百二十三` | `123` |
| `二零二四年` | `2024年` |
| `一点五` | `1.5` |

### English Text Normalization

| Input | Output |
|-------|--------|
| `$100` | `one hundred dollars` |
| `January 15, 2024` | `january fifteenth twenty twenty four` |
| `3.14` | `three point one four` |

### Japanese Text Normalization

| Input | Output |
|-------|--------|
| `100円` | `百円` |
| `2024年` | `二千二十四年` |
| `3月15日` | `三月十五日` |

## Dependencies

- [rustfst](https://github.com/Garvys/rustfst) - FST operations (Rust implementation of OpenFST)
- [thiserror](https://github.com/dtolnay/thiserror) - Error handling
- [regex](https://github.com/rust-lang/regex) - Regular expressions
- [once_cell](https://github.com/matklad/once_cell) - Lazy initialization
- [serde_json](https://github.com/serde-rs/json) - JSON parsing

## Compatibility with Python WeText

This Rust implementation is designed to be compatible with the Python [wetext](https://github.com/pengzhendong/wetext) library. The core TN/ITN functionality produces identical results for the same inputs.

**Differences from Python version:**

| Aspect | Python wetext | Rust wetext-rs |
|--------|--------------|----------------|
| Language detection | Chinese/English only | Adds Japanese detection (via Hiragana/Katakana) |
| Contractions | Runtime loaded | Compile-time embedded |
| Error handling | Python exceptions | `Result<T, WeTextError>` |
| FST library | kaldifst | rustfst |

## Credits

- Original Python implementation: [pengzhendong/wetext](https://github.com/pengzhendong/wetext)
- FST weight files: [ModelScope - pengzhendong/wetext](https://modelscope.cn/models/pengzhendong/wetext)
- WeTextProcessing grammar: [wenet-e2e/WeTextProcessing](https://github.com/wenet-e2e/WeTextProcessing)

## License

Apache-2.0
