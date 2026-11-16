use crate::parser::ParsedSource;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize)]
pub struct BurstinessAnalysis {
    pub mean_line_length: f64,
    pub std_dev_line_length: f64,
    pub line_length_cv: f64,
    pub character_entropy: f64,
    pub entropy_variance: f64,
    pub ai_probability: f64,
    pub signals: Vec<String>,
}

pub fn analyze_burstiness(parsed: &ParsedSource) -> BurstinessAnalysis {
    let mut signals = Vec::new();
    
    // Filter out empty lines, comment-only lines, and lines that are solely closing/opening delimiters
    let code_lines: Vec<&str> = parsed
        .lines
        .iter()
        .filter(|l| !l.is_empty && !l.is_comment_only)
        .map(|l| l.trimmed.as_str())
        .filter(|trimmed| !is_sole_delimiter(trimmed))
        .collect();

    let n = code_lines.len();
    if n < 4 {
        return BurstinessAnalysis {
            mean_line_length: 0.0,
            std_dev_line_length: 0.0,
            line_length_cv: 0.5,
            character_entropy: 0.0,
            entropy_variance: 0.0,
            ai_probability: 0.50,
            signals: vec!["Sample too small for burstiness analysis".to_string()],
        };
    }

    // Line length statistics
    let lengths: Vec<f64> = code_lines.iter().map(|l| l.len() as f64).collect();
    let sum: f64 = lengths.iter().sum();
    let mean_line_length = sum / n as f64;

    let variance: f64 = lengths
        .iter()
        .map(|l| {
            let diff = l - mean_line_length;
            diff * diff
        })
        .sum::<f64>()
        / n as f64;
    let std_dev_line_length = variance.sqrt();

    // Coefficient of variation CV = sigma / mu
    let line_length_cv = if mean_line_length > 0.0 {
        std_dev_line_length / mean_line_length
    } else {
        0.0
    };

    // Calculate overall character entropy
    let mut char_counts: HashMap<char, usize> = HashMap::new();
    let mut total_chars = 0;
    for line in &code_lines {
        for ch in line.chars() {
            if !ch.is_whitespace() {
                *char_counts.entry(ch).or_insert(0) += 1;
                total_chars += 1;
            }
        }
    }

    let mut character_entropy = 0.0;
    if total_chars > 0 {
        for &count in char_counts.values() {
            let p = count as f64 / total_chars as f64;
            character_entropy -= p * p.log2();
        }
    }

    // Line-level entropy across substantive lines (length >= 6)
    let mut line_entropies = Vec::new();
    for line in &code_lines {
        if line.len() < 6 {
            continue;
        }
        let mut line_char_counts: HashMap<char, usize> = HashMap::new();
        let mut line_chars = 0;
        for ch in line.chars() {
            if !ch.is_whitespace() {
                *line_char_counts.entry(ch).or_insert(0) += 1;
                line_chars += 1;
            }
        }
        if line_chars > 0 {
            let mut ent = 0.0;
            for &count in line_char_counts.values() {
                let p = count as f64 / line_chars as f64;
                ent -= p * p.log2();
            }
            line_entropies.push(ent);
        }
    }

    let ent_n = line_entropies.len();
    let (_mean_ent, entropy_variance) = if ent_n >= 4 {
        let mean_e = line_entropies.iter().sum::<f64>() / ent_n as f64;
        let var_e = line_entropies
            .iter()
            .map(|e| {
                let diff = e - mean_e;
                diff * diff
            })
            .sum::<f64>()
            / ent_n as f64;
        (mean_e, var_e)
    } else {
        (0.0, 0.0)
    };

    if line_length_cv < 0.50 {
        signals.push(format!(
            "Low line length coefficient of variation ({:.2}), showing uniform pacing characteristic of AI generation",
            line_length_cv
        ));
    } else if line_length_cv > 0.75 {
        signals.push(format!(
            "High line length burstiness ({:.2}), characteristic of human-written code variance",
            line_length_cv
        ));
    }

    if entropy_variance < 0.40 && ent_n >= 8 {
        signals.push(format!(
            "Low per-line entropy variance ({:.2}), indicating uniform predictability across lines",
            entropy_variance
        ));
    } else if entropy_variance > 0.85 && ent_n >= 8 {
        signals.push(format!(
            "High entropy burstiness ({:.2}), common in human coding with irregular idioms",
            entropy_variance
        ));
    }

    // Scoring
    let mut prob: f64 = 0.50;

    // AI code has low CV (< 0.50)
    if line_length_cv < 0.50 {
        prob += 0.15;
    } else if line_length_cv > 0.75 {
        prob -= 0.15;
    }

    // AI code has low line-level entropy variance (< 0.40)
    if entropy_variance < 0.40 && ent_n >= 8 {
        prob += 0.15;
    } else if entropy_variance > 0.85 && ent_n >= 8 {
        prob -= 0.15;
    }

    let ai_probability = prob.clamp(0.05, 0.95);

    BurstinessAnalysis {
        mean_line_length,
        std_dev_line_length,
        line_length_cv,
        character_entropy,
        entropy_variance,
        ai_probability,
        signals,
    }
}

fn is_sole_delimiter(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return true;
    }
    trimmed.chars().all(|c| "{}[];(),:".contains(c))
}
