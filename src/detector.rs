use crate::metrics::ComprehensiveMetrics;
use crate::parser::lang::Language;
use crate::parser::ParsedSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Classification {
    HighConfidenceAi,
    LikelyAi,
    MixedOrUncertain,
    LikelyHuman,
    HighConfidenceHuman,
}

impl Classification {
    pub fn label(&self) -> &'static str {
        match self {
            Classification::HighConfidenceAi => "AI-Generated (High Confidence)",
            Classification::LikelyAi => "Likely AI-Generated",
            Classification::MixedOrUncertain => "Mixed / Uncertain (Human + AI)",
            Classification::LikelyHuman => "Likely Human-Written",
            Classification::HighConfidenceHuman => "Human-Written (High Confidence)",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DetectionResult {
    pub file_path: String,
    pub language: String,
    pub lines_total: usize,
    pub lines_code: usize,
    pub lines_comment: usize,
    pub ai_probability: f64,
    pub percentage: f64,
    pub classification: Classification,
    pub confidence_level: String,
    pub metrics: ComprehensiveMetrics,
    pub signals: Vec<String>,
    pub summary: String,
}

pub struct Detector;

impl Detector {
    pub fn analyze(content: &str, file_path: &str) -> DetectionResult {
        let language = Language::from_path(file_path);
        let parsed = ParsedSource::parse(content, language);
        let metrics = ComprehensiveMetrics::compute(&parsed);

        // Collect all signals
        let mut signals = Vec::new();
        signals.extend(metrics.fingerprints.signals.clone());
        signals.extend(metrics.comments.signals.clone());
        signals.extend(metrics.identifiers.signals.clone());
        signals.extend(metrics.burstiness.signals.clone());
        signals.extend(metrics.structural.signals.clone());

        // Composite weighted score:
        // Comments: 25%, Identifiers: 25%, Burstiness: 20%, Structural: 15%, Fingerprints: 15%
        let weighted_score = (metrics.comments.ai_probability * 0.25)
            + (metrics.identifiers.ai_probability * 0.25)
            + (metrics.burstiness.ai_probability * 0.20)
            + (metrics.structural.ai_probability * 0.15)
            + (metrics.fingerprints.ai_probability * 0.15);

        // Human override penalty: if human markers or heavy abbreviations are present, clamp upper bound
        let mut final_score = weighted_score;
        if metrics.comments.human_marker_count >= 2 {
            final_score = final_score.min(0.35);
        } else if metrics.comments.human_marker_count == 1 {
            final_score = final_score.min(0.55);
        }

        // Strong AI override: if multiple explicit fingerprints are found, boost score
        if metrics.fingerprints.matched_patterns_count >= 2 {
            final_score = final_score.max(0.85);
        }

        let ai_probability = final_score.clamp(0.01, 0.99);
        let percentage = (ai_probability * 1000.0).round() / 10.0;

        let classification = if ai_probability >= 0.80 {
            Classification::HighConfidenceAi
        } else if ai_probability >= 0.60 {
            Classification::LikelyAi
        } else if ai_probability >= 0.40 {
            Classification::MixedOrUncertain
        } else if ai_probability >= 0.20 {
            Classification::LikelyHuman
        } else {
            Classification::HighConfidenceHuman
        };

        let confidence_level = if parsed.code_lines < 6 {
            "Low (insufficient lines for statistical confidence)".to_string()
        } else if parsed.code_lines < 20 {
            "Moderate (short file sample)".to_string()
        } else {
            "High (sufficient statistical sample)".to_string()
        };

        let summary = format!(
            "{} ({:.1}%) - {} [{}]",
            classification.label(),
            percentage,
            language.name(),
            confidence_level
        );

        DetectionResult {
            file_path: file_path.to_string(),
            language: language.name().to_string(),
            lines_total: parsed.total_lines,
            lines_code: parsed.code_lines,
            lines_comment: parsed.comment_lines,
            ai_probability,
            percentage,
            classification,
            confidence_level,
            metrics,
            signals,
            summary,
        }
    }
}
