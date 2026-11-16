use clap::Parser;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process;
use walkdir::WalkDir;

use ai_code_detector::detector::{DetectionResult, Detector};
use ai_code_detector::parser::lang::Language;
use ai_code_detector::report::Reporter;

#[derive(Parser, Debug)]
#[command(
    name = "ai-code-detector",
    author = "revelino01",
    version = "0.1.0",
    about = "Statistical and heuristic detector for AI-generated source code"
)]
struct Cli {
    /// Target file(s), directory, or '-' for stdin
    #[arg(default_value = ".")]
    target: String,

    /// Comma-separated list of file extensions to include (e.g. rs,py,js,ts,go,cpp)
    #[arg(short, long)]
    ext: Option<String>,

    /// Fail with exit code 1 if any file exceeds this AI probability threshold (0.0 - 1.0)
    #[arg(short, long)]
    threshold: Option<f64>,

    /// Print output in structured JSON format
    #[arg(long)]
    json: bool,

    /// Show detailed metric breakdown and detected heuristic signals
    #[arg(short, long)]
    verbose: bool,

    /// Only output minimal results
    #[arg(short, long)]
    quiet: bool,

    /// Disable recursive directory scanning
    #[arg(long)]
    no_recursive: bool,
}

fn main() {
    let args = Cli::parse();

    // Check for stdin
    if args.target == "-" {
        let mut buffer = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut buffer) {
            eprintln!("Error reading from stdin: {}", e);
            process::exit(1);
        }
        let result = Detector::analyze(&buffer, "stdin");
        handle_results(vec![result], &args);
        return;
    }

    let target_path = PathBuf::from(&args.target);
    if !target_path.exists() {
        eprintln!("Error: Target path '{}' does not exist.", args.target);
        process::exit(1);
    }

    let allowed_exts: Option<Vec<String>> = args.ext.as_ref().map(|exts| {
        exts.split(',')
            .map(|e| e.trim().trim_start_matches('.').to_lowercase())
            .collect()
    });

    let files_to_scan = collect_files(&target_path, !args.no_recursive, allowed_exts.as_deref());

    if files_to_scan.is_empty() {
        if !args.quiet && !args.json {
            println!("No supported code files found in '{}'.", args.target);
        }
        return;
    }

    let mut results = Vec::new();
    for file_path in files_to_scan {
        match fs::read_to_string(&file_path) {
            Ok(content) => {
                let path_str = file_path.to_string_lossy().to_string();
                let result = Detector::analyze(&content, &path_str);
                results.push(result);
            }
            Err(e) => {
                if !args.quiet && !args.json {
                    eprintln!("Skipping {}: {}", file_path.display(), e);
                }
            }
        }
    }

    handle_results(results, &args);
}

fn collect_files(
    root: &Path,
    recursive: bool,
    allowed_exts: Option<&[String]>,
) -> Vec<PathBuf> {
    if root.is_file() {
        return vec![root.to_path_buf()];
    }

    let max_depth = if recursive { usize::MAX } else { 1 };
    let mut files = Vec::new();

    for entry in WalkDir::new(root)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.') && name != "target" && name != "node_modules" && name != "vendor"
        })
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.is_file() {
            if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                let is_allowed = match allowed_exts {
                    Some(ext_list) => ext_list.iter().any(|allowed| allowed == &ext_lower),
                    None => Language::is_supported_extension(&ext_lower),
                };
                if is_allowed {
                    files.push(p.to_path_buf());
                }
            }
        }
    }

    files
}

fn handle_results(results: Vec<DetectionResult>, args: &Cli) {
    if args.json {
        let json_output = serde_json::to_string_pretty(&results).unwrap();
        println!("{}", json_output);
    } else {
        for res in &results {
            Reporter::print_summary(res, args.verbose);
        }
        if results.len() > 1 && !args.quiet {
            Reporter::print_batch_summary(&results);
        }
    }

    // Check threshold failure condition
    if let Some(threshold) = args.threshold {
        let violated = results.iter().any(|r| r.ai_probability >= threshold);
        if violated {
            process::exit(1);
        }
    }
}
