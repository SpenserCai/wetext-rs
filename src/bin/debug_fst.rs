// Debug script for complete normalization flow
use wetext_rs::*;

mod text_normalizer_debug {
    use rustfst::algorithms::compose::compose;
    use rustfst::algorithms::shortest_path;
    use rustfst::fst_impls::VectorFst;
    use rustfst::fst_traits::SerializableFst;
    use rustfst::prelude::*;
    use rustfst::semirings::TropicalWeight;
    use rustfst::utils::{acceptor, decode_linear_fst};
    use rustfst::{Label, EPS_LABEL};
    use std::path::Path;

    pub fn test_fst<P: AsRef<Path>>(fst_path: P, input: &str) -> Result<String, String> {
        let path = fst_path.as_ref();
        let fst = VectorFst::<TropicalWeight>::read(path)
            .map_err(|e| format!("Load error: {}", e))?;

        // Use UTF-8 bytes as labels (matching WeText FST encoding)
        let labels: Vec<Label> = input.as_bytes().iter().map(|&b| b as Label).collect();
        let input_fst: VectorFst<TropicalWeight> = acceptor(&labels, TropicalWeight::one());

        let composed: VectorFst<TropicalWeight> = compose::<
            TropicalWeight,
            VectorFst<TropicalWeight>,
            VectorFst<TropicalWeight>,
            VectorFst<TropicalWeight>,
            _,
            _,
        >(&input_fst, &fst)
        .map_err(|e| format!("Compose error: {}", e))?;

        if composed.num_states() == 0 {
            return Err(format!("[NO MATCH] {}", input));
        }

        let best_path: VectorFst<TropicalWeight> =
            shortest_path(&composed).map_err(|e| format!("Shortest path error: {}", e))?;

        if best_path.num_states() == 0 {
            return Err(format!("[NO PATH] {}", input));
        }

        let path_result =
            decode_linear_fst(&best_path).map_err(|e| format!("Decode error: {}", e))?;

        // FST output is UTF-8 bytes
        let bytes: Vec<u8> = path_result
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
        String::from_utf8(bytes).map_err(|e| format!("UTF-8 error: {}", e))
    }
}

fn main() {
    let fst_dir = "fsts";

    println!("=== Complete normalization flow test ===\n");

    // Test using Normalizer API
    let config = NormalizerConfig::new()
        .with_lang(Language::Zh)
        .with_operator(Operator::Tn);

    let mut normalizer = Normalizer::new(fst_dir, config);

    println!("1. Chinese TN test:");
    let tn_inputs = vec!["123", "2024年", "100元", "3/4", "1.5", "下午3点30分"];
    for input in &tn_inputs {
        match normalizer.normalize(input) {
            Ok(result) => println!("   '{}' => '{}'", input, result),
            Err(e) => println!("   '{}' => Error: {:?}", input, e),
        }
    }

    println!("\n2. Chinese ITN test:");
    let config_itn = NormalizerConfig::new()
        .with_lang(Language::Zh)
        .with_operator(Operator::Itn);
    let mut normalizer_itn = Normalizer::new(fst_dir, config_itn);

    let itn_inputs = vec!["一百二十三", "二零二四年", "四分之三", "一点五"];
    for input in &itn_inputs {
        match normalizer_itn.normalize(input) {
            Ok(result) => println!("   '{}' => '{}'", input, result),
            Err(e) => println!("   '{}' => Error: {:?}", input, e),
        }
    }

    // Test the fraction issue specifically
    println!("\n3. Fraction reorder test:");
    let tagger_path = format!("{}/zh/tn/tagger.fst", fst_dir);
    let verbalizer_path = format!("{}/zh/tn/verbalizer.fst", fst_dir);
    
    // Get tagger output for 3/4
    if let Ok(tagged) = text_normalizer_debug::test_fst(&tagger_path, "3/4") {
        println!("   tagger output: '{}'", tagged);
        
        // Try verbalizer with original order
        if let Ok(result) = text_normalizer_debug::test_fst(&verbalizer_path, &tagged) {
            println!("   verbalizer (original): '{}'", result);
        }
        
        // Try with reordered (denominator before numerator)
        let reordered = r#"fraction { denominator: "四" numerator: "三" } "#;
        if let Ok(result) = text_normalizer_debug::test_fst(&verbalizer_path, reordered) {
            println!("   verbalizer (reordered): '{}'", result);
        }
    }
}
