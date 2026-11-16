pub mod burstiness;
pub mod comments;
pub mod fingerprints;
pub mod identifiers;
pub mod structural;

use burstiness::{analyze_burstiness, BurstinessAnalysis};
use comments::{analyze_comments, CommentAnalysis};
use fingerprints::{analyze_fingerprints, FingerprintAnalysis};
use identifiers::{analyze_identifiers, IdentifierAnalysis};
use structural::{analyze_structural, StructuralAnalysis};

use crate::parser::ParsedSource;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ComprehensiveMetrics {
    pub comments: CommentAnalysis,
    pub identifiers: IdentifierAnalysis,
    pub burstiness: BurstinessAnalysis,
    pub structural: StructuralAnalysis,
    pub fingerprints: FingerprintAnalysis,
}

impl ComprehensiveMetrics {
    pub fn compute(parsed: &ParsedSource) -> Self {
        Self {
            comments: analyze_comments(parsed),
            identifiers: analyze_identifiers(parsed),
            burstiness: analyze_burstiness(parsed),
            structural: analyze_structural(parsed),
            fingerprints: analyze_fingerprints(parsed),
        }
    }
}
