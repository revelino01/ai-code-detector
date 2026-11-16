#!/usr/bin/env python3
"""
AI Code Detector - Statistical and Heuristic Analyzer for AI-Generated Code
A lightweight, zero-dependency Python implementation replicating the detection engine.
"""

import sys
import os
import re
import math
import json
import argparse
from pathlib import Path
from typing import Dict, List, Any, Optional

COMMON_KEYWORDS = {
    "if", "else", "for", "while", "do", "break", "continue", "return", "switch", "case",
    "default", "try", "catch", "finally", "throw", "throws", "new", "this", "super",
    "class", "interface", "struct", "enum", "type", "typedef", "import", "from", "export",
    "as", "package", "public", "private", "protected", "static", "final", "const", "let",
    "var", "fn", "function", "def", "async", "await", "yield", "true", "false", "null",
    "nil", "none", "void", "int", "float", "double", "char", "bool", "boolean", "string",
    "str", "mut", "pub", "impl", "trait", "where", "match", "use", "mod", "crate", "self",
    "Self", "pass", "elif", "in", "is", "not", "and", "or", "lambda", "global", "nonlocal",
    "with", "assert", "del", "raise", "except"
}

HUMAN_ABBREVIATIONS = {
    "i", "j", "k", "n", "m", "s", "c", "x", "y", "z", "e", "err", "ok", "v", "val",
    "res", "ret", "tmp", "buf", "len", "ptr", "idx", "cnt", "pos", "acc", "dst", "src",
    "cur", "prev", "next", "req", "ctx", "cfg", "opt", "opts", "arg", "args", "cb", "fn",
    "fp", "fd", "db", "tx", "rx", "sz", "ch", "num", "str", "vec", "arr", "elem", "attr",
    "param", "prop", "desc", "spec", "init", "auth", "info", "msg", "cmd", "pkg", "mod",
    "lib", "util", "mgr", "calc"
}

DIDACTIC_PHRASES = [
    "function to", "helper function", "utility function", "check if", "check whether",
    "calculate the", "calculates the", "compute the", "computes the", "validate the",
    "initialize the", "initializes the", "loop through", "iterate through", "iterates over",
    "traverse the", "filter the", "find the", "return the", "returns the", "base case",
    "recursive case", "step 1", "step 2", "step 3", "edge case", "handle error",
    "parse the", "example usage", "driver code", "create a new", "convert the"
]

HUMAN_MARKERS = [
    "todo", "fixme", "hack", "xxx", "bug", "workaround", "wtf", "temporary",
    "don't touch", "dont touch", "magic number", "dirty hack", "quick fix",
    "ugly", "yikes", "legacy", "deprecated because", "fucked", "damn"
]

LLM_FINGERPRINTS = [
    ("example usage", "Found 'example usage' pattern common in LLM demonstrations"),
    ("usage example", "Found 'usage example' section header"),
    ("driver code", "Found 'driver code' label typical of generated competitive/tutorial code"),
    ("test the function", "Found 'test the function' explanatory comment"),
    ("sample output", "Found 'sample output' demonstration block"),
    ("time complexity: o(", "Found explicit Big-O time complexity annotation"),
    ("space complexity: o(", "Found explicit Big-O space complexity annotation"),
    ("your_api_key", "Found standard LLM placeholder 'your_api_key'"),
    ("your-api-key", "Found standard LLM placeholder 'your-api-key'"),
    ("your_token", "Found standard LLM placeholder 'your_token'"),
    ("<your_", "Found angle-bracketed LLM template token"),
    ("replace with your", "Found instruction 'replace with your' placeholder"),
    ("here is how to use", "Found conversational instructional snippet"),
    ("```", "Found raw Markdown code fence inside source"),
]


def find_comment_start(line: str, prefix: str) -> int:
    in_single_quote = False
    in_double_quote = False
    escaped = False
    prefix_len = len(prefix)
    i = 0
    n = len(line)
    while i < n:
        c = line[i]
        if escaped:
            escaped = False
            i += 1
            continue
        if c == '\\':
            escaped = True
            i += 1
            continue
        if c == '"' and not in_single_quote:
            in_double_quote = not in_double_quote
            i += 1
            continue
        if c == "'" and not in_double_quote:
            in_single_quote = not in_single_quote
            i += 1
            continue
        if not in_single_quote and not in_double_quote:
            if line[i:i + prefix_len] == prefix:
                return i
        i += 1
    return -1


def detect_language(path_str: str) -> str:
    ext = Path(path_str).suffix.lower().lstrip(".")
    mapping = {
        "py": "Python", "pyw": "Python",
        "rs": "Rust",
        "js": "JavaScript", "jsx": "JavaScript", "mjs": "JavaScript", "cjs": "JavaScript",
        "ts": "TypeScript", "tsx": "TypeScript",
        "c": "C", "h": "C",
        "cpp": "C++", "cc": "C++", "cxx": "C++", "hpp": "C++",
        "go": "Go",
        "java": "Java",
    }
    return mapping.get(ext, "Generic")


def parse_source(content: str, language: str) -> Dict[str, Any]:
    lines = content.splitlines()
    total_lines = len(lines)
    code_lines = 0
    comment_lines = 0
    blank_lines = 0
    extracted_comments = []
    line_records = []

    in_docstring = False
    in_block_comment = False

    for idx, raw in enumerate(lines):
        trimmed = raw.strip()
        if not trimmed:
            blank_lines += 1
            line_records.append({"raw": raw, "trimmed": "", "indent": 0, "is_empty": True, "code": "", "comment": None})
            continue

        indent = len(raw) - len(raw.lstrip(" "))
        code_part = trimmed
        comment_part = None

        if language == "Python":
            if not in_docstring:
                if trimmed.startswith('"""') or trimmed.startswith("'''"):
                    sep = '"""' if trimmed.startswith('"""') else "'''"
                    if trimmed == sep or not trimmed[3:].endswith(sep):
                        in_docstring = True
                    comment_lines += 1
                    extracted_comments.append(trimmed)
                    comment_part = trimmed
                    code_part = ""
            else:
                comment_lines += 1
                extracted_comments.append(trimmed)
                comment_part = trimmed
                code_part = ""
                if '"""' in trimmed or "'''" in trimmed:
                    in_docstring = False
        else:
            if in_block_comment:
                comment_lines += 1
                extracted_comments.append(trimmed)
                comment_part = trimmed
                code_part = ""
                if "*/" in trimmed:
                    in_block_comment = False
            elif trimmed.startswith("/*"):
                comment_lines += 1
                extracted_comments.append(trimmed)
                comment_part = trimmed
                code_part = ""
                if "*/" not in trimmed[2:]:
                    in_block_comment = True

        if code_part:
            comment_prefix = "#" if language == "Python" else "//"
            if trimmed.startswith(comment_prefix):
                comment_lines += 1
                c_text = trimmed[len(comment_prefix):].strip()
                if c_text:
                    extracted_comments.append(c_text)
                    comment_part = c_text
                code_part = ""
            else:
                c_idx = find_comment_start(trimmed, comment_prefix)
                if c_idx != -1:
                    code_part = trimmed[:c_idx].strip()
                    c_text = trimmed[c_idx + len(comment_prefix):].strip()
                    if c_text:
                        extracted_comments.append(c_text)
                        comment_part = c_text
                    code_lines += 1
                else:
                    code_lines += 1

        line_records.append({
            "raw": raw,
            "trimmed": trimmed,
            "indent": indent,
            "is_empty": False,
            "code": code_part,
            "comment": comment_part
        })

    # Extract identifiers
    identifiers = []
    ident_pattern = re.compile(r'\b[a-zA-Z_][a-zA-Z0-9_]*\b')
    for rec in line_records:
        if rec["code"]:
            for match in ident_pattern.findall(rec["code"]):
                if match not in COMMON_KEYWORDS:
                    identifiers.append(match)

    return {
        "language": language,
        "lines": line_records,
        "comments": extracted_comments,
        "identifiers": identifiers,
        "total_lines": total_lines,
        "code_lines": code_lines,
        "comment_lines": comment_lines,
        "blank_lines": blank_lines,
    }


def analyze_comments(parsed: Dict[str, Any]) -> Dict[str, Any]:
    signals = []
    didactic_count = 0
    tautological_count = 0
    human_marker_count = 0
    markdown_doc_count = 0

    comments = parsed["comments"]
    total = len(comments)
    code_lines = max(1, parsed["code_lines"])
    density = parsed["comment_lines"] / code_lines

    for c in comments:
        c_lower = c.lower()
        for marker in HUMAN_MARKERS:
            if marker in c_lower:
                human_marker_count += 1
        for phrase in DIDACTIC_PHRASES:
            if phrase in c_lower:
                didactic_count += 1
                break
        if "```" in c or "###" in c or "parameters:" in c_lower or "returns:" in c_lower or "args:" in c_lower:
            markdown_doc_count += 1

    lines = parsed["lines"]
    for i in range(len(lines) - 1):
        c = lines[i]["comment"]
        next_code = lines[i + 1]["code"]
        if c and next_code:
            words = [w.lower() for w in re.findall(r'\b[a-zA-Z]{4,}\b', c)]
            if words:
                matches = sum(1 for w in words if w in next_code.lower())
                if matches / len(words) >= 0.5:
                    tautological_count += 1

    if didactic_count > 0:
        signals.append(f"Found {didactic_count} didactic/instructional comments explaining routine operations")
    if tautological_count > 0:
        signals.append(f"Found {tautological_count} tautological comments that restate adjacent identifiers")
    if markdown_doc_count > 0:
        signals.append(f"Found {markdown_doc_count} structured docstring/markdown annotations typical of LLM output")
    if human_marker_count > 0:
        signals.append(f"Detected {human_marker_count} informal/human comments (TODO, FIXME, workarounds)")

    prob = 0.40
    if total == 0:
        prob = 0.35
    else:
        if 0.15 <= density <= 0.60:
            prob += 0.15
        elif density > 0.60:
            prob += 0.20
        prob += min(0.35, (didactic_count / max(1, total)) * 0.35)
        prob += min(0.20, (tautological_count / max(1, total)) * 0.20)
        if markdown_doc_count > 0:
            prob += 0.15

    if human_marker_count > 0:
        prob -= min(0.50, human_marker_count * 0.20)

    return {
        "comment_density": density,
        "didactic_count": didactic_count,
        "tautological_count": tautological_count,
        "human_marker_count": human_marker_count,
        "markdown_doc_count": markdown_doc_count,
        "ai_probability": max(0.05, min(0.95, prob)),
        "signals": signals,
    }


def analyze_identifiers(parsed: Dict[str, Any]) -> Dict[str, Any]:
    signals = []
    idents = parsed["identifiers"]
    total = len(idents)
    if total == 0:
        return {
            "total_identifiers": 0, "avg_length": 0.0, "short_identifier_ratio": 0.0,
            "ai_probability": 0.50, "signals": ["No identifiers found to analyze"]
        }

    total_len = sum(len(i) for i in idents)
    avg_length = total_len / total
    short_count = sum(1 for i in idents if len(i) <= 3)
    short_ratio = short_count / total
    human_abbr_count = sum(1 for i in idents if i.lower() in HUMAN_ABBREVIATIONS)
    unique_human_abbrs = {i.lower() for i in idents if i.lower() in HUMAN_ABBREVIATIONS}
    descriptive_count = sum(1 for i in idents if len(i) >= 8 and ('_' in i or any(c.isupper() for c in i[1:])))
    descriptive_ratio = descriptive_count / total

    if avg_length >= 9.0:
        signals.append(f"High average identifier length ({avg_length:.1f} chars), typical of LLM descriptive naming")
    elif avg_length <= 5.5:
        signals.append(f"Terse average identifier length ({avg_length:.1f} chars), characteristic of human shorthand")

    if short_ratio <= 0.12 and total >= 10:
        signals.append(f"Very low proportion of short identifiers ({short_ratio * 100:.1f}%), consistent with AI code")
    elif short_ratio >= 0.25:
        signals.append(f"High frequency of short/terse identifiers ({short_ratio * 100:.1f}%), indicating human coding idioms")

    if descriptive_ratio >= 0.40:
        signals.append(f"Dense usage of descriptive multi-word identifiers ({descriptive_ratio * 100:.1f}%)")

    unique_abbr_count = len(unique_human_abbrs)
    if unique_abbr_count >= 3:
        sample_abbrs = ", ".join(sorted(unique_human_abbrs)[:5])
        signals.append(f"Found {unique_abbr_count} distinct human domain abbreviations ({sample_abbrs})")

    prob = 0.50
    if avg_length >= 9.0:
        prob += 0.15
    elif avg_length <= 5.8:
        prob -= 0.20

    if short_ratio < 0.12:
        prob += 0.15
    elif short_ratio > 0.25:
        prob -= 0.20

    if descriptive_ratio > 0.35:
        prob += 0.15

    if unique_abbr_count >= 5:
        prob -= 0.15
    elif unique_abbr_count >= 3:
        prob -= 0.08

    return {
        "total_identifiers": total,
        "avg_length": avg_length,
        "short_identifier_ratio": short_ratio,
        "common_human_abbr_count": human_abbr_count,
        "descriptive_names_ratio": descriptive_ratio,
        "ai_probability": max(0.05, min(0.95, prob)),
        "signals": signals,
    }


def is_sole_delimiter(s: str) -> bool:
    trimmed = s.strip()
    return not trimmed or all(c in "{}[];(),:" for c in trimmed)


def analyze_burstiness(parsed: Dict[str, Any]) -> Dict[str, Any]:
    signals = []
    code_lines = [
        l["trimmed"] for l in parsed["lines"]
        if not l["is_empty"] and l["code"] and not is_sole_delimiter(l["trimmed"])
    ]
    n = len(code_lines)
    if n < 4:
        return {
            "mean_line_length": 0.0, "line_length_cv": 0.5, "entropy_variance": 0.0,
            "ai_probability": 0.50, "signals": ["Sample too small for burstiness analysis"]
        }

    lengths = [len(l) for l in code_lines]
    mean_len = sum(lengths) / n
    variance = sum((l - mean_len) ** 2 for l in lengths) / n
    std_dev = math.sqrt(variance)
    cv = (std_dev / mean_len) if mean_len > 0 else 0.0

    # Line entropies for lines with >= 6 chars
    line_entropies = []
    for line in code_lines:
        if len(line) < 6:
            continue
        counts = {}
        non_ws = [c for c in line if not c.isspace()]
        if not non_ws:
            continue
        for c in non_ws:
            counts[c] = counts.get(c, 0) + 1
        ent = 0.0
        tot = len(non_ws)
        for cnt in counts.values():
            p = cnt / tot
            ent -= p * math.log2(p)
        line_entropies.append(ent)

    ent_n = len(line_entropies)
    if ent_n >= 4:
        mean_ent = sum(line_entropies) / ent_n
        ent_var = sum((e - mean_ent) ** 2 for e in line_entropies) / ent_n
    else:
        ent_var = 0.0

    if cv < 0.50:
        signals.append(f"Low line length coefficient of variation ({cv:.2f}), showing uniform pacing characteristic of AI generation")
    elif cv > 0.75:
        signals.append(f"High line length burstiness ({cv:.2f}), characteristic of human-written code variance")

    if ent_var < 0.40 and ent_n >= 8:
        signals.append(f"Low per-line entropy variance ({ent_var:.2f}), indicating uniform predictability across lines")
    elif ent_var > 0.85 and ent_n >= 8:
        signals.append(f"High entropy burstiness ({ent_var:.2f}), common in human coding with irregular idioms")

    prob = 0.50
    if cv < 0.50:
        prob += 0.15
    elif cv > 0.75:
        prob -= 0.15

    if ent_var < 0.40 and ent_n >= 8:
        prob += 0.15
    elif ent_var > 0.85 and ent_n >= 8:
        prob -= 0.15

    return {
        "mean_line_length": mean_len,
        "line_length_cv": cv,
        "entropy_variance": ent_var,
        "ai_probability": max(0.05, min(0.95, prob)),
        "signals": signals,
    }


def analyze_structural(parsed: Dict[str, Any]) -> Dict[str, Any]:
    signals = []
    lines = [l for l in parsed["lines"] if not l["is_empty"]]
    if not lines:
        return {"indentation_uniformity": 1.0, "ai_probability": 0.5, "signals": []}

    indents = [l["indent"] for l in lines if l["indent"] > 0]
    total_indented = len(indents)
    c2_only = sum(1 for i in indents if i % 2 == 0 and i % 4 != 0)
    step = 2 if c2_only > 0 else 4

    matching = sum(1 for i in indents if i % step == 0)
    uniformity = (matching / total_indented) if total_indented > 0 else 1.0

    nestings = [l["indent"] // step for l in lines]
    max_nesting = max(nestings) if nestings else 0

    guards = 0
    for l in lines:
        t = l["trimmed"]
        if t.startswith("if ") or t.startswith("if("):
            if any(k in t for k in ("return", "continue", "break", "raise", "throw")):
                guards += 1

    if uniformity >= 0.98 and total_indented >= 10:
        signals.append(f"Pristine indentation uniformity ({uniformity * 100:.0f}% {step} spaces), typical of clean LLM code generation")
    elif uniformity < 0.85:
        signals.append(f"Irregular indentation uniformity ({uniformity * 100:.0f}%), characteristic of manual editing")

    if max_nesting <= 3 and len(lines) >= 20:
        signals.append(f"Controlled shallow nesting (max depth {max_nesting}), common in synthesized code")
    elif max_nesting >= 5:
        signals.append(f"Deep nesting level (depth {max_nesting}), characteristic of complex human logic")

    prob = 0.50
    if uniformity > 0.95:
        prob += 0.10
    elif uniformity < 0.85:
        prob -= 0.15

    if max_nesting <= 3 and len(lines) >= 20:
        prob += 0.10
    elif max_nesting >= 5:
        prob -= 0.15

    if guards >= 2:
        prob += 0.10

    return {
        "indentation_uniformity": uniformity,
        "max_nesting_level": max_nesting,
        "guard_clause_count": guards,
        "ai_probability": max(0.05, min(0.95, prob)),
        "signals": signals,
    }


def analyze_fingerprints(content: str) -> Dict[str, Any]:
    signals = []
    matches = []
    lower = content.lower()

    for pattern, desc in LLM_FINGERPRINTS:
        if pattern in lower:
            matches.append(pattern)
            signals.append(desc)

    textbook = ["def binary_search(", "def quick_sort(", "def merge_sort(", "function quicksort("]
    for sig in textbook:
        if sig in lower:
            matches.append(sig)
            signals.append("Detected canonical textbook algorithm template signature")
            break

    prob = 0.50
    if matches:
        prob += min(0.40, len(matches) * 0.20)

    return {
        "matched_patterns_count": len(matches),
        "matches": matches,
        "ai_probability": max(0.05, min(0.95, prob)),
        "signals": signals,
    }


def detect(content: str, file_path: str = "snippet") -> Dict[str, Any]:
    language = detect_language(file_path)
    parsed = parse_source(content, language)

    m_comments = analyze_comments(parsed)
    m_idents = analyze_identifiers(parsed)
    m_burst = analyze_burstiness(parsed)
    m_struct = analyze_structural(parsed)
    m_fingers = analyze_fingerprints(content)

    all_signals = m_fingers["signals"] + m_comments["signals"] + m_idents["signals"] + m_burst["signals"] + m_struct["signals"]

    weighted_score = (
        (m_comments["ai_probability"] * 0.25)
        + (m_idents["ai_probability"] * 0.25)
        + (m_burst["ai_probability"] * 0.20)
        + (m_struct["ai_probability"] * 0.15)
        + (m_fingers["ai_probability"] * 0.15)
    )

    if m_comments["human_marker_count"] >= 2:
        final_score = min(0.35, weighted_score)
    elif m_comments["human_marker_count"] == 1:
        final_score = min(0.55, weighted_score)
    else:
        final_score = weighted_score

    if m_fingers["matched_patterns_count"] >= 2:
        final_score = max(0.85, final_score)

    is_empty_sample = parsed["code_lines"] == 0 and parsed["comment_lines"] == 0
    if is_empty_sample:
        ai_prob = 0.0
        pct = 0.0
        cls_name = "LikelyHuman"
        label = "Likely Human-Written"
    else:
        ai_prob = max(0.01, min(0.99, final_score))
        pct = round(ai_prob * 100, 1)

        if ai_prob >= 0.80:
            cls_name = "HighConfidenceAi"
            label = "AI-Generated (High Confidence)"
        elif ai_prob >= 0.60:
            cls_name = "LikelyAi"
            label = "Likely AI-Generated"
        elif ai_prob >= 0.40:
            cls_name = "MixedOrUncertain"
            label = "Mixed / Uncertain (Human + AI)"
        elif ai_prob >= 0.20:
            cls_name = "LikelyHuman"
            label = "Likely Human-Written"
        else:
            cls_name = "HighConfidenceHuman"
            label = "Human-Written (High Confidence)"

    return {
        "file_path": file_path,
        "language": language,
        "lines_total": parsed["total_lines"],
        "lines_code": parsed["code_lines"],
        "lines_comment": parsed["comment_lines"],
        "ai_probability": ai_prob,
        "percentage": pct,
        "classification": cls_name,
        "label": label,
        "signals": all_signals,
    }


def main():
    parser = argparse.ArgumentParser(description="Statistical and heuristic detector for AI-generated source code")
    parser.add_argument("target", nargs="?", default=".", help="Target file or directory (or '-' for stdin)")
    parser.add_argument("-e", "--ext", help="Comma-separated list of file extensions to include (e.g. rs,py,js,ts,go,cpp)")
    parser.add_argument("--json", action="store_true", help="Output in JSON format")
    parser.add_argument("-v", "--verbose", action="store_true", help="Verbose details")
    parser.add_argument("-t", "--threshold", type=float, help="Fail threshold (0.0 - 1.0)")
    args = parser.parse_args()

    allowed_exts = None
    if args.ext:
        allowed_exts = {e.strip().lstrip(".").lower() for e in args.ext.split(",") if e.strip()}

    supported_exts = {
        "py", "pyw", "rs", "js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx",
        "c", "h", "cpp", "cxx", "cc", "hpp", "hxx", "go", "java"
    }

    results = []
    if args.target == "-":
        content = sys.stdin.read()
        fake_name = "stdin"
        if args.ext:
            first_ext = args.ext.split(",")[0].strip().lstrip(".")
            if first_ext:
                fake_name = f"stdin.{first_ext}"
        results.append(detect(content, fake_name))
    else:
        target_path = Path(args.target)
        if not target_path.exists():
            sys.stderr.write(f"Error: Target path '{args.target}' does not exist.\n")
            sys.exit(1)

        if target_path.is_file():
            ext = target_path.suffix.lower().lstrip(".")
            if allowed_exts is None or ext in allowed_exts:
                content = target_path.read_text(encoding="utf-8", errors="ignore")
                results.append(detect(content, str(target_path)))
        elif target_path.is_dir():
            for root, _, files in os.walk(target_path):
                if any(ignored in root for ignored in [".git", "target", "node_modules", "vendor"]):
                    continue
                for f in files:
                    fp = Path(root) / f
                    ext = fp.suffix.lower().lstrip(".")
                    is_match = (ext in allowed_exts) if allowed_exts is not None else (ext in supported_exts)
                    if is_match:
                        try:
                            content = fp.read_text(encoding="utf-8", errors="ignore")
                            results.append(detect(content, str(fp)))
                        except Exception:
                            pass

    if args.json:
        print(json.dumps(results, indent=2))
    else:
        badge_map = {
            "HighConfidenceAi": "[ HIGH AI ]",
            "LikelyAi": "[ LIKELY AI ]",
            "MixedOrUncertain": "[ MIXED/UNCERTAIN ]",
            "LikelyHuman": "[ LIKELY HUMAN ]",
            "HighConfidenceHuman": "[ HIGH HUMAN ]",
        }
        for r in results:
            badge = badge_map.get(r["classification"], f"[{r['label']}]")
            print(f"{badge}\t{r['file_path']}\t({r['percentage']}% AI probability)")
            if args.verbose:
                for s in r["signals"]:
                    print(f"  • {s}")
                print()

        if len(results) > 1:
            total = len(results)
            ai_count = sum(1 for r in results if r["classification"] in {"HighConfidenceAi", "LikelyAi"})
            human_count = sum(1 for r in results if r["classification"] in {"LikelyHuman", "HighConfidenceHuman"})
            mixed_count = sum(1 for r in results if r["classification"] == "MixedOrUncertain")
            print("--------------------------------------------------")
            print(f"Scan Summary: {total} files analyzed")
            print(f"  ● Likely AI: {ai_count} ({(ai_count / total) * 100:.1f}%)")
            print(f"  ● Likely Human: {human_count} ({(human_count / total) * 100:.1f}%)")
            print(f"  ● Mixed / Uncertain: {mixed_count} ({(mixed_count / total) * 100:.1f}%)")
            print("--------------------------------------------------")

    if args.threshold is not None:
        if any(r["ai_probability"] >= args.threshold for r in results):
            sys.exit(1)


if __name__ == "__main__":
    main()
