use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    C,
    Cpp,
    Go,
    Java,
    Generic,
}

impl Language {
    pub fn name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::C => "C",
            Language::Cpp => "C++",
            Language::Go => "Go",
            Language::Java => "Java",
            Language::Generic => "Generic",
        }
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let p = path.as_ref();
        if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "rs" => Language::Rust,
                "py" | "pyw" => Language::Python,
                "js" | "mjs" | "cjs" | "jsx" => Language::JavaScript,
                "ts" | "mts" | "cts" | "tsx" => Language::TypeScript,
                "c" | "h" => Language::C,
                "cpp" | "cxx" | "cc" | "hpp" | "hxx" => Language::Cpp,
                "go" => Language::Go,
                "java" => Language::Java,
                _ => Language::Generic,
            }
        } else {
            Language::Generic
        }
    }

    pub fn line_comment_prefix(&self) -> &'static [&'static str] {
        match self {
            Language::Python => &["#"],
            Language::Rust => &["///", "//! ", "//"],
            Language::JavaScript | Language::TypeScript | Language::C | Language::Cpp | Language::Go | Language::Java => {
                &["//"]
            }
            Language::Generic => &["//", "#"],
        }
    }

    pub fn is_supported_extension(ext: &str) -> bool {
        matches!(
            ext.to_lowercase().as_str(),
            "rs" | "py" | "pyw" | "js" | "mjs" | "cjs" | "jsx" | "ts" | "mts" | "cts" | "tsx" | "c" | "h" | "cpp" | "cxx" | "cc" | "hpp" | "hxx" | "go" | "java"
        )
    }
}
