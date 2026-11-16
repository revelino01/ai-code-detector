import unittest
from pathlib import Path
import sys

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent))
from detector import detect, detect_language


class TestAiCodeDetectorPython(unittest.TestCase):

    def test_language_detection(self):
        self.assertEqual(detect_language("test.rs"), "Rust")
        self.assertEqual(detect_language("test.py"), "Python")
        self.assertEqual(detect_language("index.js"), "JavaScript")
        self.assertEqual(detect_language("types.ts"), "TypeScript")
        self.assertEqual(detect_language("kernel.c"), "C")
        self.assertEqual(detect_language("algo.cpp"), "C++")
        self.assertEqual(detect_language("main.go"), "Go")
        self.assertEqual(detect_language("App.java"), "Java")

    def test_ai_python_quicksort(self):
        p = Path(__file__).parent.parent / "tests" / "fixtures" / "ai_samples" / "python_quicksort.py"
        content = p.read_text(encoding="utf-8")
        result = detect(content, str(p))
        self.assertGreaterEqual(result["ai_probability"], 0.70)
        self.assertIn("Ai", result["classification"])

    def test_ai_rust_service(self):
        p = Path(__file__).parent.parent / "tests" / "fixtures" / "ai_samples" / "rust_service.rs"
        content = p.read_text(encoding="utf-8")
        result = detect(content, str(p))
        self.assertGreaterEqual(result["ai_probability"], 0.60)
        self.assertIn("Ai", result["classification"])

    def test_human_c_kernel(self):
        p = Path(__file__).parent.parent / "tests" / "fixtures" / "human_samples" / "c_kernel_style.c"
        content = p.read_text(encoding="utf-8")
        result = detect(content, str(p))
        self.assertLessEqual(result["ai_probability"], 0.35)
        self.assertIn("Human", result["classification"])

    def test_human_python_script(self):
        p = Path(__file__).parent.parent / "tests" / "fixtures" / "human_samples" / "python_script.py"
        content = p.read_text(encoding="utf-8")
        result = detect(content, str(p))
        self.assertLessEqual(result["ai_probability"], 0.40)
        self.assertIn("Human", result["classification"])

    def test_empty_content(self):
        result = detect("", "empty.py")
        self.assertEqual(result["lines_total"], 0)
        self.assertEqual(result["lines_code"], 0)
        self.assertEqual(result["ai_probability"], 0.0)

    def test_utf8_multibyte_comments(self):
        src = 'greeting = "你好，世界 🦀" # say hello\n'
        result = detect(src, "test.py")
        self.assertEqual(result["lines_code"], 1)
        self.assertEqual(result["lines_total"], 1)

    def test_url_not_parsed_as_comment(self):
        src = 'url = "https://example.com/api"\ncolor = "#123456"\n'
        result = detect(src, "test.py")
        self.assertEqual(result["lines_comment"], 0)
        self.assertEqual(result["lines_code"], 2)

    def test_two_space_indentation(self):
        src = "function main() {\n  let a = 1;\n  if (a > 0) {\n    let b = 2;\n    return b;\n  }\n  return a;\n}\n"
        result = detect(src, "test.js")
        self.assertEqual(result["lines_code"], 8)


if __name__ == "__main__":
    unittest.main()
