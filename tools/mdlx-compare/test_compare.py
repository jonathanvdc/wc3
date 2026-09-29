"""Runner failure-path tests; do not require WhiteoutLib or network access."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location("compare", Path(__file__).with_name("compare.py"))
compare = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(compare)


class ComparisonTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def executable(self, name, code):
        path = self.root / name
        path.write_text("#!/usr/bin/env python3\n" + code)
        path.chmod(0o755)
        return path

    def test_success_without_output_is_not_agreement(self):
        executable = self.executable("empty", "pass\n")
        result = compare.stage(executable, self.root / "input", self.root / "output", "hive", 5)
        self.assertEqual(result["status"], "missing_output")

    def test_timeout_retains_diagnostics(self):
        executable = self.executable("slow", "import sys, time\nprint('before timeout', file=sys.stderr, flush=True)\ntime.sleep(5)\n")
        result = compare.stage(executable, self.root / "input", self.root / "output", "hive", 0.2)
        self.assertEqual(result["status"], "timeout")
        self.assertIn("before timeout", result["diagnostics"])

    def test_oracle_runs_even_if_our_conversion_fails(self):
        ours = self.executable("ours", "import sys\nprint('conversion refused', file=sys.stderr)\nsys.exit(1)\n")
        oracle = self.executable("oracle", "import sys\nfrom pathlib import Path\nPath(sys.argv[2]).write_text('Version { FormatVersion 800, } Model \\\"Example\\\" {}')\n")
        source = self.root / "input.mdx"
        source.write_bytes(b"MDLX")
        output = self.root / "results"
        output.mkdir()
        result = compare.process(source, Path(source.name), output, ours, oracle, "hive", 5)
        self.assertEqual(result["stages"]["oracle_conversion"]["status"], "ok")
        self.assertEqual(result["status"], "review")
        self.assertTrue((output / "oracle.mdl").exists())
        self.assertTrue((output / "report.json").exists())

    def test_disagreement_retains_a_field_level_diff(self):
        left, right, diff = [self.root / name for name in ["ours.mdl", "oracle.mdl", "conversion.diff"]]
        left.write_text('static Alpha 0.5,\n')
        right.write_text('static Alpha 1.0,\n')
        result = compare.compare(left, right, diff)
        self.assertEqual(result["status"], "canonical_difference")
        self.assertIn('-static Alpha 0.5,', diff.read_text())
        self.assertIn('+static Alpha 1.0,', diff.read_text())

    def test_parser_issues_cannot_be_silent_agreement(self):
        executable = self.executable("warning", "import sys\nfrom pathlib import Path\nprint('unsupported data', file=sys.stderr)\nPath(sys.argv[2]).write_text('same canonical text')\n")
        source = self.root / "input.mdx"
        source.write_bytes(b"MDLX")
        output = self.root / "results"
        output.mkdir()
        result = compare.process(source, Path(source.name), output, executable, executable, "engine", 5)
        self.assertEqual(result["status"], "review")
        self.assertIn("oracle_conversion:parser_issues", result["findings"])
        self.assertEqual(result["comparisons"]["independent_conversions"]["status"], "equal")


if __name__ == "__main__":
    unittest.main()
