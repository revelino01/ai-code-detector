use crate::parser::ParsedSource;

#[derive(Debug, Clone, serde::Serialize)]
pub struct CommentAnalysis {
    pub comment_density: f64,
    pub didactic_count: usize,
    pub tautological_count: usize,
    pub human_marker_count: usize,
    pub markdown_doc_count: usize,
    pub ai_probability: f64,
    pub signals: Vec<String>,
}

pub fn analyze_comments(parsed: &ParsedSource) -> CommentAnalysis {
    let mut signals = Vec::new();
    let mut didactic_count = 0;
    let mut tautological_count = 0;
    let mut human_marker_count = 0;
    let mut markdown_doc_count = 0;

    let total_comments = parsed.comments.len();
    let code_lines = parsed.code_lines.max(1);
    let comment_density = parsed.comment_lines as f64 / code_lines as f64;

    let didactic_phrases = [
        "function to",
        "helper function",
        "utility function",
        "check if",
        "check whether",
        "calculate the",
        "calculates the",
        "compute the",
        "computes the",
        "validate the",
        "initialize the",
        "initializes the",
        "loop through",
        "iterate through",
        "iterates over",
        "traverse the",
        "filter the",
        "find the",
        "return the",
        "returns the",
        "base case",
        "recursive case",
        "step 1",
        "step 2",
        "step 3",
        "edge case",
        "handle error",
        "parse the",
        "example usage",
        "driver code",
        "create a new",
        "convert the",
    ];

    let human_markers = [
        "todo",
        "fixme",
        "hack",
        "xxx",
        "bug",
        "workaround",
        "wtf",
        "temporary",
        "don't touch",
        "dont touch",
        "magic number",
        "dirty hack",
        "quick fix",
        "ugly",
        "yikes",
        "legacy",
        "deprecated because",
        "fucked",
        "damn",
    ];

    // Analyze individual comments
    for comment in &parsed.comments {
        let lower = comment.to_lowercase();

        // Check human markers
        for marker in human_markers {
            if lower.contains(marker) {
                human_marker_count += 1;
            }
        }

        // Check didactic phrases
        for phrase in didactic_phrases {
            if lower.contains(phrase) {
                didactic_count += 1;
                break;
            }
        }

        // Check markdown documentation patterns typical of LLMs
        if comment.contains("```")
            || comment.contains("###")
            || lower.contains("parameters:")
            || lower.contains("returns:")
            || lower.contains("raises:")
            || lower.contains("args:")
            || lower.contains("time complexity:")
            || lower.contains("space complexity:")
        {
            markdown_doc_count += 1;
        }
    }

    // Check tautological overlap (comment describing the immediate next code line)
    for i in 0..parsed.lines.len() {
        let line = &parsed.lines[i];
        if let Some(ref comment) = line.comment_part {
            // Check next line code part
            if i + 1 < parsed.lines.len() {
                let next_code = &parsed.lines[i + 1].code_part;
                if !next_code.is_empty() {
                    if is_tautological(comment, next_code) {
                        tautological_count += 1;
                    }
                }
            }
        }
    }

    // Signals collection
    if didactic_count > 0 {
        signals.push(format!(
            "Found {} didactic/instructional comments explaining routine operations",
            didactic_count
        ));
    }
    if tautological_count > 0 {
        signals.push(format!(
            "Found {} tautological comments that restate adjacent identifiers",
            tautological_count
        ));
    }
    if markdown_doc_count > 0 {
        signals.push(format!(
            "Found {} structured docstring/markdown annotations typical of LLM output",
            markdown_doc_count
        ));
    }
    if human_marker_count > 0 {
        signals.push(format!(
            "Detected {} informal/human comments (TODO, FIXME, workarounds)",
            human_marker_count
        ));
    }

    // Probability scoring
    let mut prob = 0.40; // baseline neutral

    // Comment density contribution: AI commonly writes 15% - 50% comments
    if total_comments == 0 {
        // No comments at all is moderately more common in human code
        prob = 0.35;
    } else {
        if comment_density >= 0.15 && comment_density <= 0.60 {
            prob += 0.15;
        } else if comment_density > 0.60 {
            prob += 0.20;
        }

        let didactic_ratio = didactic_count as f64 / total_comments.max(1) as f64;
        prob += (didactic_ratio * 0.35).min(0.35);

        let taut_ratio = tautological_count as f64 / total_comments.max(1) as f64;
        prob += (taut_ratio * 0.20).min(0.20);

        if markdown_doc_count > 0 {
            prob += 0.15;
        }
    }

    // Heavy human penalty
    if human_marker_count > 0 {
        let penalty = (human_marker_count as f64 * 0.20).min(0.50);
        prob -= penalty;
    }

    let ai_probability = prob.clamp(0.05, 0.95);

    CommentAnalysis {
        comment_density,
        didactic_count,
        tautological_count,
        human_marker_count,
        markdown_doc_count,
        ai_probability,
        signals,
    }
}

fn is_tautological(comment: &str, next_code: &str) -> bool {
    let comment_words: Vec<String> = comment
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 3)
        .map(|w| w.to_lowercase())
        .collect();

    if comment_words.is_empty() {
        return false;
    }

    let code_lower = next_code.to_lowercase();
    let mut matches = 0;
    for word in &comment_words {
        if code_lower.contains(word) {
            matches += 1;
        }
    }

    let match_ratio = matches as f64 / comment_words.len() as f64;
    match_ratio >= 0.50
}
