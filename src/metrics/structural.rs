use crate::parser::ParsedSource;

#[derive(Debug, Clone, serde::Serialize)]
pub struct StructuralAnalysis {
    pub indentation_uniformity: f64,
    pub detected_indent_step: usize,
    pub max_nesting_level: usize,
    pub avg_nesting_level: f64,
    pub guard_clause_count: usize,
    pub consecutive_blank_lines_count: usize,
    pub ai_probability: f64,
    pub signals: Vec<String>,
}

pub fn analyze_structural(parsed: &ParsedSource) -> StructuralAnalysis {
    let mut signals = Vec::new();

    let non_empty_lines: Vec<&crate::parser::ParsedLine> =
        parsed.lines.iter().filter(|l| !l.is_empty).collect();

    if non_empty_lines.is_empty() {
        return StructuralAnalysis {
            indentation_uniformity: 1.0,
            detected_indent_step: 4,
            max_nesting_level: 0,
            avg_nesting_level: 0.0,
            guard_clause_count: 0,
            consecutive_blank_lines_count: 0,
            ai_probability: 0.50,
            signals: vec!["Insufficient lines for structural analysis".to_string()],
        };
    }

    // Determine common indent step (usually 2 or 4 spaces)
    let mut step_counts_2_only = 0usize;
    for line in &non_empty_lines {
        let spaces = line.indent_spaces;
        if spaces > 0 && spaces % 2 == 0 && spaces % 4 != 0 {
            step_counts_2_only += 1;
        }
    }
    // Any line with 2, 6, 10, 14... spaces indicates a 2-space indentation step
    let detected_indent_step = if step_counts_2_only > 0 {
        2
    } else {
        4
    };

    // Calculate indent compliance
    let mut matching_indents = 0;
    let mut total_indented_lines = 0;
    let mut nesting_levels = Vec::new();
    let mut guard_clause_count = 0;

    for line in &non_empty_lines {
        let spaces = line.indent_spaces;
        if spaces > 0 {
            total_indented_lines += 1;
            if spaces % detected_indent_step == 0 {
                matching_indents += 1;
            }
        }

        let nesting = spaces / detected_indent_step;
        nesting_levels.push(nesting);

        // Check for guard clauses (early return/continue/break)
        let trimmed = line.trimmed.as_str();
        if trimmed.starts_with("if ")
            || trimmed.starts_with("if(")
            || (trimmed.starts_with("if let ") && trimmed.ends_with('}'))
        {
            if trimmed.contains("return")
                || trimmed.contains("continue")
                || trimmed.contains("break")
                || trimmed.contains("throw")
                || trimmed.contains("raise")
                || trimmed.contains("panic!")
            {
                guard_clause_count += 1;
            }
        }
    }

    let indentation_uniformity = if total_indented_lines > 0 {
        matching_indents as f64 / total_indented_lines as f64
    } else {
        1.0
    };

    let max_nesting_level = nesting_levels.iter().cloned().max().unwrap_or(0);
    let avg_nesting_level = if !nesting_levels.is_empty() {
        nesting_levels.iter().sum::<usize>() as f64 / nesting_levels.len() as f64
    } else {
        0.0
    };

    // Count consecutive blank lines (> 1 blank line in a row)
    let mut consecutive_blank_lines_count = 0;
    let mut current_blank_streak = 0;
    for line in &parsed.lines {
        if line.is_empty {
            current_blank_streak += 1;
            if current_blank_streak > 1 {
                consecutive_blank_lines_count += 1;
            }
        } else {
            current_blank_streak = 0;
        }
    }

    if indentation_uniformity >= 0.98 && total_indented_lines >= 10 {
        signals.push(format!(
            "Pristine indentation uniformity ({:.0}% {} spaces), typical of clean LLM code generation",
            indentation_uniformity * 100.0,
            detected_indent_step
        ));
    } else if indentation_uniformity < 0.85 {
        signals.push(format!(
            "Irregular indentation uniformity ({:.0}%), characteristic of manual editing",
            indentation_uniformity * 100.0
        ));
    }

    if max_nesting_level <= 3 && non_empty_lines.len() >= 25 {
        signals.push(format!(
            "Controlled shallow nesting (max depth {}), common in synthesized code",
            max_nesting_level
        ));
    } else if max_nesting_level >= 5 {
        signals.push(format!(
            "Deep nesting level (depth {}), characteristic of complex human logic",
            max_nesting_level
        ));
    }

    if guard_clause_count >= 2 {
        signals.push(format!(
            "Frequent guard clause / early exit patterns ({} detected)",
            guard_clause_count
        ));
    }

    if consecutive_blank_lines_count >= 3 {
        signals.push(format!(
            "Found multiple irregular blank line clusters ({} occurrences), suggesting human edits",
            consecutive_blank_lines_count
        ));
    }

    // Scoring
    let mut prob: f64 = 0.50;

    if indentation_uniformity > 0.95 {
        prob += 0.10;
    } else if indentation_uniformity < 0.85 {
        prob -= 0.15;
    }

    if max_nesting_level <= 3 && non_empty_lines.len() >= 20 {
        prob += 0.10;
    } else if max_nesting_level >= 5 {
        prob -= 0.15;
    }

    if guard_clause_count >= 2 {
        prob += 0.10;
    }

    if consecutive_blank_lines_count >= 2 {
        prob -= 0.10;
    }

    let ai_probability = prob.clamp(0.05, 0.95);

    StructuralAnalysis {
        indentation_uniformity,
        detected_indent_step,
        max_nesting_level,
        avg_nesting_level,
        guard_clause_count,
        consecutive_blank_lines_count,
        ai_probability,
        signals,
    }
}
