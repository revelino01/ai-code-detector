use crate::parser::ParsedSource;

#[derive(Debug, Clone, serde::Serialize)]
pub struct IdentifierAnalysis {
    pub total_identifiers: usize,
    pub unique_identifiers: usize,
    pub avg_length: f64,
    pub short_identifier_ratio: f64,
    pub common_human_abbr_count: usize,
    pub descriptive_names_ratio: f64,
    pub casing_uniformity: f64,
    pub ai_probability: f64,
    pub signals: Vec<String>,
}

pub fn analyze_identifiers(parsed: &ParsedSource) -> IdentifierAnalysis {
    let mut signals = Vec::new();
    let total = parsed.identifiers.len();

    if total == 0 {
        return IdentifierAnalysis {
            total_identifiers: 0,
            unique_identifiers: 0,
            avg_length: 0.0,
            short_identifier_ratio: 0.0,
            common_human_abbr_count: 0,
            descriptive_names_ratio: 0.0,
            casing_uniformity: 1.0,
            ai_probability: 0.50,
            signals: vec!["No identifiers found to analyze".to_string()],
        };
    }

    let mut unique_set = std::collections::HashSet::new();
    let mut total_len = 0;
    let mut short_count = 0;
    let mut human_abbr_count = 0;
    let mut descriptive_count = 0;

    let mut snake_count = 0;
    let mut camel_count = 0;

    const HUMAN_ABBREVIATIONS: &[&str] = &[
        "i", "j", "k", "n", "m", "s", "c", "x", "y", "z", "e", "err", "ok", "v", "val",
        "res", "ret", "tmp", "buf", "len", "ptr", "idx", "cnt", "pos", "acc", "dst", "src",
        "cur", "prev", "next", "req", "ctx", "cfg", "opt", "opts", "arg", "args", "cb", "fn",
        "fp", "fd", "db", "tx", "rx", "sz", "ch", "num", "str", "vec", "arr", "elem", "attr",
        "param", "prop", "desc", "spec", "init", "auth", "info", "msg", "cmd", "pkg", "mod",
        "lib", "util", "mgr", "calc",
    ];

    for ident in &parsed.identifiers {
        unique_set.insert(ident.as_str());
        let len = ident.len();
        total_len += len;

        let lower = ident.to_lowercase();
        if len <= 3 {
            short_count += 1;
        }

        if HUMAN_ABBREVIATIONS.contains(&lower.as_str()) {
            human_abbr_count += 1;
        }

        // Multi-word descriptive identifiers typical of LLM output (e.g. is_valid_user, calculate_total)
        if (ident.contains('_') && ident.len() >= 8) || (has_camel_case(ident) && ident.len() >= 8) {
            descriptive_count += 1;
        }

        // Check casing style
        if ident.contains('_') && !has_camel_case(ident) {
            snake_count += 1;
        } else if has_camel_case(ident) && !ident.contains('_') {
            camel_count += 1;
        }
    }

    let avg_length = total_len as f64 / total as f64;
    let short_identifier_ratio = short_count as f64 / total as f64;
    let descriptive_names_ratio = descriptive_count as f64 / total as f64;

    let classified_casing = snake_count + camel_count;
    let casing_uniformity = if classified_casing > 0 {
        let max_style = snake_count.max(camel_count);
        max_style as f64 / classified_casing as f64
    } else {
        1.0
    };

    if avg_length >= 9.0 {
        signals.push(format!(
            "High average identifier length ({:.1} chars), typical of LLM descriptive naming",
            avg_length
        ));
    } else if avg_length <= 5.5 {
        signals.push(format!(
            "Terse average identifier length ({:.1} chars), characteristic of human shorthand",
            avg_length
        ));
    }

    if short_identifier_ratio <= 0.12 && total >= 10 {
        signals.push(format!(
            "Very low proportion of short identifiers ({:.1}%), consistent with AI code",
            short_identifier_ratio * 100.0
        ));
    } else if short_identifier_ratio >= 0.25 {
        signals.push(format!(
            "High frequency of short/terse identifiers ({:.1}%), indicating human coding idioms",
            short_identifier_ratio * 100.0
        ));
    }

    if descriptive_names_ratio >= 0.40 {
        signals.push(format!(
            "Dense usage of descriptive multi-word identifiers ({:.1}%)",
            descriptive_names_ratio * 100.0
        ));
    }

    if human_abbr_count >= 5 {
        signals.push(format!(
            "Found {} common human domain abbreviations (ctx, req, res, buf, etc.)",
            human_abbr_count
        ));
    }

    // Scoring
    let mut prob: f64 = 0.50;

    // Length scoring: AI centers around 8.5 - 13.0
    if avg_length >= 9.0 {
        prob += 0.15;
    } else if avg_length <= 5.8 {
        prob -= 0.20;
    }

    // Short identifier ratio: AI is < 0.12, Human is > 0.22
    if short_identifier_ratio < 0.12 {
        prob += 0.15;
    } else if short_identifier_ratio > 0.25 {
        prob -= 0.20;
    }

    // Descriptive naming ratio
    if descriptive_names_ratio > 0.35 {
        prob += 0.15;
    }

    // Human abbreviations
    let abbr_ratio = human_abbr_count as f64 / total as f64;
    if abbr_ratio > 0.15 {
        prob -= 0.15;
    }

    // High casing uniformity adds slight AI probability
    if casing_uniformity > 0.95 && classified_casing >= 10 {
        prob += 0.05;
    }

    let ai_probability = prob.clamp(0.05, 0.95);

    IdentifierAnalysis {
        total_identifiers: total,
        unique_identifiers: unique_set.len(),
        avg_length,
        short_identifier_ratio,
        common_human_abbr_count: human_abbr_count,
        descriptive_names_ratio,
        casing_uniformity,
        ai_probability,
        signals,
    }
}

fn has_camel_case(s: &str) -> bool {
    let mut has_lower = false;
    let mut has_upper_after_lower = false;

    for ch in s.chars() {
        if ch.is_lowercase() {
            has_lower = true;
        } else if ch.is_uppercase() && has_lower {
            has_upper_after_lower = true;
            break;
        }
    }
    has_upper_after_lower
}
