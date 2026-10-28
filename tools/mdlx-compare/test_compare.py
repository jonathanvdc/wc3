"""Runner failure-path tests; do not require WhiteoutLib or network access."""
import importlib.util
import contextlib
import io
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

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

    def test_invalid_execution_limits_are_rejected_before_work(self):
        for command, option, values, diagnostic in (
            (["run"], "--timeout", ["0", "-1", "nan", "inf", "-inf", "1e999", "invalid"],
             "--timeout"),
            (["build", "--source", str(self.root)], "--jobs", ["0", "-1", "1.5", "invalid"],
             "--jobs"),
        ):
            for value in values:
                with self.subTest(option=option, value=value), patch.object(
                        compare.sys, "argv", ["compare.py", *command, f"{option}={value}"]), patch.object(
                        compare, "build") as build, patch.object(compare, "run") as run:
                    errors = io.StringIO()
                    with contextlib.redirect_stderr(errors), self.assertRaises(SystemExit) as error:
                        compare.main()
                    self.assertEqual(error.exception.code, 2)
                    self.assertIn(diagnostic, errors.getvalue())
                    build.assert_not_called()
                    run.assert_not_called()

    def test_positive_execution_limits_reach_the_selected_command(self):
        for command, option, value, expected in (
            (["run"], "timeout", "0.25", 0.25),
            (["build", "--source", str(self.root)], "jobs", "2", 2),
        ):
            with self.subTest(option=option), patch.object(
                    compare.sys, "argv", ["compare.py", *command, f"--{option}={value}"]), patch.object(
                    compare, command[0], return_value=0) as selected:
                self.assertEqual(compare.main(), 0)
                self.assertEqual(getattr(selected.call_args.args[0], option), expected)

    def test_success_without_output_is_not_agreement(self):
        executable = self.executable("empty", "pass\n")
        result = compare.stage(executable, self.root / "input", self.root / "output", "hive", 5)
        self.assertEqual(result["status"], "missing_output")

    def test_repeated_dialects_produce_one_report_each_in_selection_order(self):
        executable = self.executable("converter", "import sys\nfrom pathlib import Path\nPath(sys.argv[2]).write_text('same canonical text')\n")
        build = self.root / "build"
        build.mkdir()
        oracle = build / "whiteout-adapter"
        oracle.write_bytes(executable.read_bytes())
        oracle.chmod(0o755)
        compare.save_json(build / "adapter.json", {
            **compare.LOCK, "executable_sha256": compare.digest(oracle),
            "adapter_source_sha256": compare.digest(compare.HERE / "adapter.cpp"),
            "cmake_source_sha256": compare.digest(compare.HERE / "CMakeLists.txt"),
        })
        corpus = self.root / "corpus"
        corpus.mkdir()
        (corpus / "input.mdx").write_bytes(b"MDLX")
        results = self.root / "results"
        args = SimpleNamespace(build_dir=build, corpus=corpus, results=results,
                               dialects=["hive", "engine", "hive"], timeout=5, fail_on_review=True)
        metadata = json.dumps({"target_directory": str(self.root)})
        target = self.root / "debug/examples"
        target.mkdir(parents=True)
        (target / "mdlx_compare").symlink_to(executable)
        real_run = compare.subprocess.run
        def without_build(command, **kwargs):
            if command[0] != "cargo":
                return real_run(command, **kwargs)
        with patch.object(compare.subprocess, "run", side_effect=without_build), patch.object(
                compare, "checked", side_effect=[metadata, "test-revision", ""]):
            self.assertEqual(compare.run(args), 0)
        summary = json.loads((results / "summary.json").read_text())
        self.assertEqual(summary["counts"], {"agreement": 2})
        self.assertEqual([report["dialect"] for report in summary["reports"]], ["hive", "engine"])
        self.assertEqual(json.loads((results / "run.json").read_text())["dialects"], ["hive", "engine"])
        self.assertTrue((results / "README.md").is_file())

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
