use ai_code_detector::detector::{Classification, Detector};
use ai_code_detector::parser::lang::Language;
use ai_code_detector::parser::ParsedSource;

#[test]
fn test_language_detection() {
    assert_eq!(Language::from_path("foo/bar.rs"), Language::Rust);
    assert_eq!(Language::from_path("script.py"), Language::Python);
    assert_eq!(Language::from_path("web/app.tsx"), Language::TypeScript);
    assert_eq!(Language::from_path("web/index.js"), Language::JavaScript);
    assert_eq!(Language::from_path("main.go"), Language::Go);
    assert_eq!(Language::from_path("engine.cpp"), Language::Cpp);
    assert_eq!(Language::from_path("driver.c"), Language::C);
    assert_eq!(Language::from_path("App.java"), Language::Java);
    assert_eq!(Language::from_path("notes.txt"), Language::Generic);
}

#[test]
fn test_ai_python_quicksort_detected() {
    let content = std::fs::read_to_string("tests/fixtures/ai_samples/python_quicksort.py")
        .expect("read python quicksort fixture");
    let result = Detector::analyze(&content, "tests/fixtures/ai_samples/python_quicksort.py");

    assert!(
        result.ai_probability >= 0.70,
        "Expected AI probability >= 0.70, got {:.2}",
        result.ai_probability
    );
    assert!(
        matches!(
            result.classification,
            Classification::HighConfidenceAi | Classification::LikelyAi
        ),
        "Classification was {:?}",
        result.classification
    );
    assert!(result.signals.iter().any(|s| s.contains("didactic") || s.contains("example usage") || s.contains("textbook")));
}

#[test]
fn test_ai_rust_service_detected() {
    let content = std::fs::read_to_string("tests/fixtures/ai_samples/rust_service.rs")
        .expect("read rust service fixture");
    let result = Detector::analyze(&content, "tests/fixtures/ai_samples/rust_service.rs");

    assert!(
        result.ai_probability >= 0.65,
        "Expected AI probability >= 0.65, got {:.2}",
        result.ai_probability
    );
    assert!(
        matches!(
            result.classification,
            Classification::HighConfidenceAi | Classification::LikelyAi
        ),
        "Classification was {:?}",
        result.classification
    );
}

#[test]
fn test_human_c_kernel_classified() {
    let content = std::fs::read_to_string("tests/fixtures/human_samples/c_kernel_style.c")
        .expect("read c kernel fixture");
    let result = Detector::analyze(&content, "tests/fixtures/human_samples/c_kernel_style.c");

    assert!(
        result.ai_probability <= 0.35,
        "Expected AI probability <= 0.35, got {:.2}",
        result.ai_probability
    );
    assert!(
        matches!(
            result.classification,
            Classification::LikelyHuman | Classification::HighConfidenceHuman
        ),
        "Classification was {:?}",
        result.classification
    );
    assert!(result.signals.iter().any(|s| s.contains("TODO") || s.contains("informal")));
}

#[test]
fn test_human_python_classified() {
    let content = std::fs::read_to_string("tests/fixtures/human_samples/python_script.py")
        .expect("read python script fixture");
    let result = Detector::analyze(&content, "tests/fixtures/human_samples/python_script.py");

    assert!(
        result.ai_probability <= 0.40,
        "Expected AI probability <= 0.40, got {:.2}",
        result.ai_probability
    );
    assert!(
        matches!(
            result.classification,
            Classification::LikelyHuman | Classification::HighConfidenceHuman
        ),
        "Classification was {:?}",
        result.classification
    );
}

#[test]
fn test_edge_case_empty_file() {
    let result = Detector::analyze("", "empty.rs");
    assert_eq!(result.lines_total, 0);
    assert_eq!(result.lines_code, 0);
    assert!(result.confidence_level.contains("insufficient"));
}

#[test]
fn test_edge_case_single_line() {
    let result = Detector::analyze("let x = 42;", "test.rs");
    assert_eq!(result.lines_total, 1);
    assert_eq!(result.lines_code, 1);
    assert!(result.confidence_level.contains("insufficient"));
}

#[test]
fn test_edge_case_comments_only() {
    let content = "// Just a comment\n// Another line of comment\n";
    let result = Detector::analyze(content, "comments.rs");
    assert_eq!(result.lines_code, 0);
    assert_eq!(result.lines_comment, 2);
    assert!(result.confidence_level.contains("insufficient"));
}

#[test]
fn test_json_serialization() {
    let result = Detector::analyze("fn add(a: i32, b: i32) -> i32 { a + b }", "add.rs");
    let json_str = serde_json::to_string(&result);
    assert!(json_str.is_ok());
    let unwrapped = json_str.unwrap();
    assert!(unwrapped.contains("\"file_path\":\"add.rs\""));
    assert!(unwrapped.contains("\"ai_probability\""));
}

#[test]
fn test_comment_and_identifier_parsing() {
    let src = r#"
    // Helper function to calculate sum
    fn calculate_sum(item_list: Vec<i32>) -> i32 {
        let mut total_accumulated_sum = 0;
        for i in item_list {
            total_accumulated_sum += i;
        }
        total_accumulated_sum
    }
    "#;
    let parsed = ParsedSource::parse(src, Language::Rust);
    assert_eq!(parsed.comments.len(), 1);
    assert!(parsed.comments[0].contains("Helper function"));
    assert!(parsed.identifiers.contains(&"calculate_sum".to_string()));
    assert!(parsed.identifiers.contains(&"total_accumulated_sum".to_string()));
}
