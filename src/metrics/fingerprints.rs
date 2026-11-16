use crate::parser::ParsedSource;

#[derive(Debug, Clone, serde::Serialize)]
pub struct FingerprintAnalysis {
    pub matched_patterns_count: usize,
    pub matches: Vec<String>,
    pub ai_probability: f64,
    pub signals: Vec<String>,
}

pub fn analyze_fingerprints(parsed: &ParsedSource) -> FingerprintAnalysis {
    let mut signals = Vec::new();
    let mut matches = Vec::new();

    let llm_fingerprints = [
        ("example usage", "Found 'example usage' pattern common in LLM demonstrations"),
        ("usage example", "Found 'usage example' section header"),
        ("driver code", "Found 'driver code' label typical of generated competitive/tutorial code"),
        ("test the function", "Found 'test the function' explanatory comment"),
        ("sample output", "Found 'sample output' demonstration block"),
        ("time complexity: o(", "Found explicit Big-O time complexity annotation"),
        ("space complexity: o(", "Found explicit Big-O space complexity annotation"),
        ("your_api_key", "Found standard LLM placeholder 'your_api_key'"),
        ("your-api-key", "Found standard LLM placeholder 'your-api-key'"),
        ("your_token", "Found standard LLM placeholder 'your_token'"),
        ("<your_", "Found angle-bracketed LLM template token"),
        ("replace with your", "Found instruction 'replace with your' placeholder"),
        ("here is how to use", "Found conversational conversational instructional snippet"),
        ("```", "Found raw Markdown code fence inside source"),
    ];

    let full_text = parsed
        .lines
        .iter()
        .map(|l| l.raw.as_str())
        .collect::<Vec<&str>>()
        .join("\n")
        .to_lowercase();

    for (pattern, desc) in llm_fingerprints {
        if full_text.contains(pattern) {
            matches.push(pattern.to_string());
            signals.push(desc.to_string());
        }
    }

    // Check for textbook algorithmic signatures
    let textbook_signatures = [
        "def binary_search(arr",
        "def quick_sort(arr",
        "def merge_sort(arr",
        "def is_palindrome(s",
        "fn binary_search(arr",
        "function binarysearch(",
        "function quicksort(",
    ];

    for sig in textbook_signatures {
        if full_text.contains(sig) {
            let label = "Detected canonical textbook algorithm template signature";
            matches.push(sig.to_string());
            signals.push(label.to_string());
            break;
        }
    }

    let matched_patterns_count = matches.len();

    // Scoring
    let mut prob = 0.50;
    if matched_patterns_count >= 1 {
        let boost = (matched_patterns_count as f64 * 0.20).min(0.40);
        prob += boost;
    }

    let ai_probability = prob.clamp(0.05, 0.95);

    FingerprintAnalysis {
        matched_patterns_count,
        matches,
        ai_probability,
        signals,
    }
}
