use crate::detector::{Classification, DetectionResult};

pub struct Reporter;

impl Reporter {
    pub fn print_summary(result: &DetectionResult, verbose: bool) {
        let (color_code, badge) = match result.classification {
            Classification::HighConfidenceAi => ("\x1b[1;31m", "[ HIGH AI ]"),
            Classification::LikelyAi => ("\x1b[1;33m", "[ LIKELY AI ]"),
            Classification::MixedOrUncertain => ("\x1b[1;36m", "[ MIXED/UNCERTAIN ]"),
            Classification::LikelyHuman => ("\x1b[1;32m", "[ LIKELY HUMAN ]"),
            Classification::HighConfidenceHuman => ("\x1b[1;32m", "[ HIGH HUMAN ]"),
        };
        let reset = "\x1b[0m";
        let bold = "\x1b[1m";
        let dim = "\x1b[2m";

        println!(
            "{}{}{}\t{}{}\t({:.1}% AI probability){}",
            color_code, badge, reset, bold, result.file_path, result.percentage, reset
        );

        if verbose {
            println!("{}├─ Language:{} {}", dim, reset, result.language);
            println!(
                "{}├─ Lines:{} {} total ({} code, {} comments)",
                dim, reset, result.lines_total, result.lines_code, result.lines_comment
            );
            println!("{}├─ Confidence:{} {}", dim, reset, result.confidence_level);

            println!("{}├─ Metrics Breakdown:{}", dim, reset);
            println!(
                "{}│  ├─ Comments Didactic/Tautological:{} {:.1}% AI score",
                dim, reset, result.metrics.comments.ai_probability * 100.0
            );
            println!(
                "{}│  ├─ Identifier Naming Style:{} {:.1}% AI score",
                dim, reset, result.metrics.identifiers.ai_probability * 100.0
            );
            println!(
                "{}│  ├─ Line Burstiness & Entropy:{} {:.1}% AI score",
                dim, reset, result.metrics.burstiness.ai_probability * 100.0
            );
            println!(
                "{}│  ├─ Structural Uniformity:{} {:.1}% AI score",
                dim, reset, result.metrics.structural.ai_probability * 100.0
            );
            println!(
                "{}│  └─ Fingerprint Signatures:{} {:.1}% AI score",
                dim, reset, result.metrics.fingerprints.ai_probability * 100.0
            );

            if !result.signals.is_empty() {
                println!("{}└─ Detected Signals:{}", dim, reset);
                for signal in &result.signals {
                    println!("{}   •{} {}", color_code, reset, signal);
                }
            } else {
                println!("{}└─ Detected Signals:{} None", dim, reset);
            }
            println!();
        }
    }

    pub fn print_batch_summary(results: &[DetectionResult]) {
        let bold = "\x1b[1m";
        let reset = "\x1b[0m";
        let dim = "\x1b[2m";

        let total = results.len();
        if total == 0 {
            println!("No supported code files found.");
            return;
        }

        let mut ai_count = 0;
        let mut human_count = 0;
        let mut mixed_count = 0;

        for r in results {
            match r.classification {
                Classification::HighConfidenceAi | Classification::LikelyAi => ai_count += 1,
                Classification::LikelyHuman | Classification::HighConfidenceHuman => human_count += 1,
                Classification::MixedOrUncertain => mixed_count += 1,
            }
        }

        println!("--------------------------------------------------");
        println!("{}Scan Summary:{} {} files analyzed", bold, reset, total);
        println!(
            "  \x1b[31m● Likely AI:\x1b[0m {} ({:.1}%)",
            ai_count,
            (ai_count as f64 / total as f64) * 100.0
        );
        println!(
            "  \x1b[32m● Likely Human:\x1b[0m {} ({:.1}%)",
            human_count,
            (human_count as f64 / total as f64) * 100.0
        );
        println!(
            "  \x1b[36m● Mixed / Uncertain:\x1b[0m {} ({:.1}%)",
            mixed_count,
            (mixed_count as f64 / total as f64) * 100.0
        );
        println!("{}--------------------------------------------------{}", dim, reset);
    }
}
