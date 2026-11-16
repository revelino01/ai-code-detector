pub mod lang;

use lang::Language;

#[derive(Debug, Clone)]
pub struct ParsedLine {
    pub line_number: usize,
    pub raw: String,
    pub trimmed: String,
    pub indent_spaces: usize,
    pub is_empty: bool,
    pub is_comment_only: bool,
    pub code_part: String,
    pub comment_part: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedSource {
    pub language: Language,
    pub lines: Vec<ParsedLine>,
    pub comments: Vec<String>,
    pub identifiers: Vec<String>,
    pub total_lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
}

impl ParsedSource {
    pub fn parse(content: &str, language: Language) -> Self {
        let mut lines = Vec::new();
        let mut comments = Vec::new();
        let mut total_lines = 0;
        let mut code_lines = 0;
        let mut comment_lines = 0;
        let mut blank_lines = 0;

        let raw_lines: Vec<&str> = content.lines().collect();
        let mut in_block_comment = false;
        let mut in_python_docstring = false;
        let mut python_docstring_char = "";

        for (idx, line_str) in raw_lines.iter().enumerate() {
            total_lines += 1;
            let line_number = idx + 1;
            let trimmed = line_str.trim();

            if trimmed.is_empty() {
                blank_lines += 1;
                lines.push(ParsedLine {
                    line_number,
                    raw: line_str.to_string(),
                    trimmed: String::new(),
                    indent_spaces: 0,
                    is_empty: true,
                    is_comment_only: false,
                    code_part: String::new(),
                    comment_part: None,
                });
                continue;
            }

            // Calculate indentation (converting tab to 4 spaces)
            let mut indent_spaces = 0;
            for ch in line_str.chars() {
                if ch == ' ' {
                    indent_spaces += 1;
                } else if ch == '\t' {
                    indent_spaces += 4;
                } else {
                    break;
                }
            }

            // Handle multi-line comments & Python docstrings
            if language == Language::Python {
                if !in_python_docstring {
                    if trimmed.starts_with("\"\"\"") {
                        if trimmed == "\"\"\"" || !trimmed[3..].contains("\"\"\"") {
                            in_python_docstring = true;
                            python_docstring_char = "\"\"\"";
                        }
                        comment_lines += 1;
                        comments.push(trimmed.to_string());
                        lines.push(ParsedLine {
                            line_number,
                            raw: line_str.to_string(),
                            trimmed: trimmed.to_string(),
                            indent_spaces,
                            is_empty: false,
                            is_comment_only: true,
                            code_part: String::new(),
                            comment_part: Some(trimmed.to_string()),
                        });
                        continue;
                    } else if trimmed.starts_with("'''") {
                        if trimmed == "'''" || !trimmed[3..].contains("'''") {
                            in_python_docstring = true;
                            python_docstring_char = "'''";
                        }
                        comment_lines += 1;
                        comments.push(trimmed.to_string());
                        lines.push(ParsedLine {
                            line_number,
                            raw: line_str.to_string(),
                            trimmed: trimmed.to_string(),
                            indent_spaces,
                            is_empty: false,
                            is_comment_only: true,
                            code_part: String::new(),
                            comment_part: Some(trimmed.to_string()),
                        });
                        continue;
                    }
                } else {
                    comment_lines += 1;
                    comments.push(trimmed.to_string());
                    if trimmed.ends_with(python_docstring_char) || trimmed.contains(python_docstring_char) {
                        in_python_docstring = false;
                    }
                    lines.push(ParsedLine {
                        line_number,
                        raw: line_str.to_string(),
                        trimmed: trimmed.to_string(),
                        indent_spaces,
                        is_empty: false,
                        is_comment_only: true,
                        code_part: String::new(),
                        comment_part: Some(trimmed.to_string()),
                    });
                    continue;
                }
            } else {
                // C-style block comments
                if in_block_comment {
                    comment_lines += 1;
                    comments.push(trimmed.to_string());
                    if trimmed.contains("*/") {
                        in_block_comment = false;
                    }
                    lines.push(ParsedLine {
                        line_number,
                        raw: line_str.to_string(),
                        trimmed: trimmed.to_string(),
                        indent_spaces,
                        is_empty: false,
                        is_comment_only: true,
                        code_part: String::new(),
                        comment_part: Some(trimmed.to_string()),
                    });
                    continue;
                } else if trimmed.starts_with("/*") {
                    comment_lines += 1;
                    comments.push(trimmed.to_string());
                    if !trimmed[2..].contains("*/") {
                        in_block_comment = true;
                    }
                    lines.push(ParsedLine {
                        line_number,
                        raw: line_str.to_string(),
                        trimmed: trimmed.to_string(),
                        indent_spaces,
                        is_empty: false,
                        is_comment_only: true,
                        code_part: String::new(),
                        comment_part: Some(trimmed.to_string()),
                    });
                    continue;
                }
            }

            // Single line comments
            let comment_prefixes = language.line_comment_prefix();
            let mut is_comment_only = false;
            let mut code_part = trimmed.to_string();
            let mut comment_part = None;

            for &pfx in comment_prefixes {
                if trimmed.starts_with(pfx) {
                    is_comment_only = true;
                    comment_lines += 1;
                    let comment_text = trimmed[pfx.len()..].trim().to_string();
                    comments.push(comment_text.clone());
                    comment_part = Some(comment_text);
                    code_part = String::new();
                    break;
                }
            }

            if !is_comment_only {
                // Check for inline comments
                for &pfx in comment_prefixes {
                    if let Some(pos) = find_comment_start(trimmed, pfx) {
                        let code = trimmed[..pos].trim();
                        let comment = trimmed[pos + pfx.len()..].trim();
                        if !comment.is_empty() {
                            comments.push(comment.to_string());
                            comment_part = Some(comment.to_string());
                        }
                        code_part = code.to_string();
                        break;
                    }
                }
                code_lines += 1;
            }

            lines.push(ParsedLine {
                line_number,
                raw: line_str.to_string(),
                trimmed: trimmed.to_string(),
                indent_spaces,
                is_empty: false,
                is_comment_only,
                code_part,
                comment_part,
            });
        }

        // Extract identifier tokens
        let mut identifiers = Vec::new();
        for line in &lines {
            if !line.code_part.is_empty() {
                extract_identifiers(&line.code_part, language, &mut identifiers);
            }
        }

        Self {
            language,
            lines,
            comments,
            identifiers,
            total_lines,
            code_lines,
            comment_lines,
            blank_lines,
        }
    }
}

/// Helper to find comment prefix outside string literals
fn find_comment_start(line: &str, prefix: &str) -> Option<usize> {
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

    let mut iter = line.char_indices().peekable();
    while let Some((byte_idx, ch)) = iter.next() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            continue;
        }
        if ch == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            continue;
        }

        if !in_single_quote && !in_double_quote {
            if line[byte_idx..].starts_with(prefix) {
                return Some(byte_idx);
            }
        }
    }
    None
}

/// Extracts identifier candidates, filtering out standard keywords
fn extract_identifiers(code: &str, _language: Language, out: &mut Vec<String>) {
    let mut current = String::new();
    for ch in code.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            current.push(ch);
        } else {
            if !current.is_empty() {
                if is_identifier_candidate(&current) {
                    out.push(current.clone());
                }
                current.clear();
            }
        }
    }
    if !current.is_empty() && is_identifier_candidate(&current) {
        out.push(current);
    }
}

fn is_identifier_candidate(s: &str) -> bool {
    // Must start with letter or underscore
    let first = s.chars().next().unwrap_or(' ');
    if !first.is_alphabetic() && first != '_' {
        return false;
    }
    // Filter common language keywords
    !is_common_keyword(s)
}

fn is_common_keyword(s: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        // Common across languages
        "if", "else", "for", "while", "do", "break", "continue", "return", "switch", "case",
        "default", "try", "catch", "finally", "throw", "throws", "new", "this", "super",
        "class", "interface", "struct", "enum", "type", "typedef", "import", "from", "export",
        "as", "package", "public", "private", "protected", "static", "final", "const", "let",
        "var", "fn", "function", "def", "async", "await", "yield", "true", "false", "null",
        "nil", "none", "void", "int", "float", "double", "char", "bool", "boolean", "string",
        "str", "mut", "pub", "impl", "trait", "where", "match", "use", "mod", "crate", "self",
        "Self", "pass", "elif", "in", "is", "not", "and", "or", "lambda", "global", "nonlocal",
        "with", "assert", "del", "raise", "except",
    ];
    KEYWORDS.contains(&s)
}
