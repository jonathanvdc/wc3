#!/usr/bin/env python3
"""Optional differential MDL conversion runner; Python standard library only."""
import argparse
from collections import Counter
from datetime import datetime, timezone
import difflib
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
LOCK = json.loads((HERE / "whiteout.lock.json").read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save_json(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n")


def checked(command, **kwargs):
    return subprocess.check_output(command, text=True, **kwargs).strip()


def build(args):
    source = args.source.resolve()
    revision = checked(["git", "-C", str(source), "rev-parse", "HEAD"])
    if revision != LOCK["revision"]:
        raise ValueError(f"WhiteoutLib revision {revision} differs from pin {LOCK['revision']}")
    if checked(["git", "-C", str(source), "status", "--porcelain"]):
        raise ValueError("WhiteoutLib checkout must be clean")
    directory = args.build_dir.resolve()
    subprocess.run(["cmake", "-S", str(HERE), "-B", str(directory), "-G", "Ninja",
                    *args.cmake_arg, f"-DWHITEOUT_SOURCE={source}", "-DCMAKE_BUILD_TYPE=Release"], check=True)
    subprocess.run(["cmake", "--build", str(directory), "--target", "whiteout-adapter",
                    "-j", str(args.jobs)], check=True)
    if checked(["git", "-C", str(source), "rev-parse", "HEAD"]) != revision or checked(
            ["git", "-C", str(source), "status", "--porcelain"]):
        raise ValueError("WhiteoutLib source changed during build")
    executable = directory / "whiteout-adapter"
    save_json(directory / "adapter.json", {
        **LOCK, "executable_sha256": digest(executable),
        "adapter_source_sha256": digest(HERE / "adapter.cpp"),
        "cmake_source_sha256": digest(HERE / "CMakeLists.txt"),
        "parser_mode": "PreserveOriginal", "cmake_arguments": args.cmake_arg,
    })


def stage(executable, source, output, dialect, timeout):
    command = [str(executable), str(source), str(output), dialect]
    try:
        result = subprocess.run(command, capture_output=True, timeout=timeout)
        status = "ok" if result.returncode == 0 else "failed"
        if status == "ok" and (not output.is_file() or output.stat().st_size == 0):
            status = "missing_output"
        phase = next((name for prefix, name in [(b"MDX decode:", "mdx_decode"),
                      (b"MDL parse:", "mdl_parse"), (b"MDL write:", "mdl_write")]
                      if result.stderr.startswith(prefix)), None)
        return {"status": status, "failure_phase": phase if status != "ok" else None, "exit_code": result.returncode,
                "stdout": result.stdout.decode("utf-8", errors="replace"),
                "diagnostics": result.stderr.decode("utf-8", errors="replace"),
                "command": command}
    except subprocess.TimeoutExpired as error:
        return {"status": "timeout", "command": command,
                "diagnostics": (error.stderr or b"").decode("utf-8", errors="replace")}
    except OSError as error:
        return {"status": "launch_failed", "command": command, "diagnostics": str(error)}


def compare(left, right, diff_path):
    # This is deliberately conservative: source ordering of records/tracks is
    # significant here. A difference requests review, not a verdict of data loss.
    if left.read_bytes() == right.read_bytes():
        return {"status": "equal"}
    a, b = left.read_text().splitlines(keepends=True), right.read_text().splitlines(keepends=True)
    diff = difflib.unified_diff(a, b, fromfile=left.name, tofile=right.name)
    with diff_path.open("w") as output:
        for line in diff:
            output.write(line)
    return {"status": "canonical_difference", "diff": diff_path.name}


def version(data):
    offset = 4
    while offset + 8 <= len(data):
        tag, size = data[offset:offset + 4], struct.unpack_from("<I", data, offset + 4)[0]
        if offset + 8 + size > len(data):
            return None
        if tag == b"VERS" and size >= 4:
            return struct.unpack_from("<I", data, offset + 8)[0]
        offset += 8 + size
    return None


def process(source, relative, destination, ours, oracle, dialect, timeout):
    data = source.read_bytes()
    report = {"source": str(relative), "input_sha256": hashlib.sha256(data).hexdigest(),
              "input_bytes": len(data), "version": version(data), "dialect": dialect,
              "stages": {}, "comparisons": {}}
    stages = report["stages"]
    def run(name, executable, input_name, output_name):
        stages[name] = stage(executable, input_name, destination / output_name, dialect, timeout)
        return stages[name]["status"] == "ok"
    ours_ok = run("ours_conversion", ours, source, "ours.mdl")
    oracle_ok = run("oracle_conversion", oracle, source, "oracle.mdl")
    ours_read = ours_ok and run("ours_reads_ours", ours, destination / "ours.mdl", "ours.canonical.mdl")
    oracle_read = oracle_ok and run("ours_reads_oracle", ours, destination / "oracle.mdl", "oracle.canonical.mdl")
    cross_read = ours_ok and run("oracle_reads_ours", oracle, destination / "ours.mdl", "ours.oracle-reparsed.mdl")
    cross_canonical = cross_read and run("ours_reads_cross_output", ours,
                                        destination / "ours.oracle-reparsed.mdl", "ours.cross.canonical.mdl")
    oracle_self = oracle_ok and run("oracle_reads_oracle", oracle, destination / "oracle.mdl", "oracle.reparsed.mdl")
    oracle_self_canonical = oracle_self and run("ours_reads_oracle_self_output", ours,
                                               destination / "oracle.reparsed.mdl", "oracle.self.canonical.mdl")
    comparisons = report["comparisons"]
    if ours_read:
        comparisons["ours_stability"] = compare(destination / "ours.mdl", destination / "ours.canonical.mdl",
                                                destination / "ours-stability.diff")
    if ours_read and oracle_read:
        comparisons["independent_conversions"] = compare(destination / "ours.canonical.mdl",
            destination / "oracle.canonical.mdl", destination / "conversion.diff")
    if ours_read and cross_canonical:
        comparisons["oracle_cross_read"] = compare(destination / "ours.canonical.mdl",
            destination / "ours.cross.canonical.mdl", destination / "cross-read.diff")
    if oracle_read and oracle_self_canonical:
        comparisons["oracle_stability"] = compare(destination / "oracle.canonical.mdl",
            destination / "oracle.self.canonical.mdl", destination / "oracle-stability.diff")
    findings = []
    for name, result in stages.items():
        if result["status"] != "ok":
            findings.append(f"{name}:{result['status']}")
        elif name.startswith("oracle_") and result.get("diagnostics", "").strip():
            findings.append(f"{name}:parser_issues")
    findings.extend(f"{name}:{result['status']}" for name, result in comparisons.items()
                    if result["status"] != "equal")
    if digest(source) != report["input_sha256"]:
        findings.append("input_changed_during_run")
    report["findings"] = findings
    report["status"] = "review" if findings else "agreement"
    save_json(destination / "report.json", report)
    return report


def run(args):
    directory = args.build_dir.resolve()
    metadata = json.loads((directory / "adapter.json").read_text())
    oracle = directory / "whiteout-adapter"
    expected = {**LOCK, "executable_sha256": digest(oracle),
                "adapter_source_sha256": digest(HERE / "adapter.cpp"),
                "cmake_source_sha256": digest(HERE / "CMakeLists.txt")}
    if any(metadata.get(key) != value for key, value in expected.items()):
        raise ValueError("Adapter metadata does not match pin/source/binary; rebuild it")
    corpus = args.corpus.resolve()
    sources = sorted(p for p in corpus.rglob("*") if p.is_file() and p.suffix.lower() == ".mdx")
    if not sources:
        raise ValueError("Corpus contains no MDX models")
    results = args.results.resolve() if args.results else ROOT / "target/mdlx-compare/results" / datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")
    if results.exists():
        raise ValueError(f"Results directory already exists: {results}")
    subprocess.run(["cargo", "build", "-p", "wc3", "--example", "mdlx_compare"], cwd=ROOT, check=True)
    cargo_metadata = json.loads(checked(["cargo", "metadata", "--format-version=1", "--no-deps"], cwd=ROOT))
    ours = Path(cargo_metadata["target_directory"]) / "debug/examples/mdlx_compare"
    results.mkdir(parents=True, exist_ok=False)
    save_json(results / "run.json", {
        "oracle": metadata, "ours_executable_sha256": digest(ours),
        "repository_revision": checked(["git", "rev-parse", "HEAD"], cwd=ROOT),
        "repository_dirty": bool(checked(["git", "status", "--porcelain"], cwd=ROOT)),
        "corpus": str(corpus), "dialects": args.dialects, "timeout_seconds": args.timeout,
        "comparison": "MDL decoded and re-encoded by wc3; conservative canonical text equality",
    })
    reports = []
    for index, source in enumerate(sources, 1):
        relative = source.relative_to(corpus)
        identifier = f"{index:03}-{digest(source)[:12]}"
        for dialect in args.dialects:
            destination = results / identifier / dialect
            destination.mkdir(parents=True)
            try:
                report = process(source, relative, destination, ours, oracle, dialect, args.timeout)
            except Exception as error:
                report = {"source": str(relative), "dialect": dialect, "status": "runner_error",
                          "findings": [str(error)]}
                save_json(destination / "report.json", report)
            reports.append({**report, "report": str((destination / "report.json").relative_to(results))})
            print(f"[{index}/{len(sources)}] {dialect}: {relative}: {report['status']}", flush=True)
    counts = dict(Counter(r["status"] for r in reports))
    save_json(results / "summary.json", {"counts": counts, "models": len(sources), "reports": reports})
    lines = ["# MDL compatibility report", "", f"WhiteoutLib: `{LOCK['revision']}`", "",
             f"Models: {len(sources)}; comparisons: {len(reports)}; outcomes: `{counts}`", "",
             "Differences require investigation; neither converter is treated as authoritative.", "",
             "| Model | Version | Dialect | Outcome | Findings |", "|---|---:|---|---|---|"]
    for report in reports:
        name = report["source"].replace("|", "\\|")
        findings = "; ".join(report["findings"]).replace("|", "\\|").replace("\n", " ") or "—"
        lines.append(f"| {name} | {report.get('version', '?')} | {report['dialect']} | "
                     f"[{report['status']}]({report['report']}) | {findings} |")
    (results / "README.md").write_text("\n".join(lines) + "\n")
    print(f"Results: {results}\nOutcomes: {counts}")
    if any(r["status"] == "runner_error" for r in reports):
        return 2
    return 1 if args.fail_on_review and any(r["status"] != "agreement" for r in reports) else 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    default_build = ROOT / "target/mdlx-compare/build"
    setup = commands.add_parser("build", help="build a verified pinned WhiteoutLib checkout")
    setup.add_argument("--source", type=Path, required=True)
    setup.add_argument("--build-dir", type=Path, default=default_build)
    setup.add_argument("--jobs", type=int, default=4)
    setup.add_argument("--cmake-arg", action="append", default=[], help="additional CMake argument (use --cmake-arg=-D...)")
    execute = commands.add_parser("run", help="compare every MDX in a local corpus")
    execute.add_argument("--corpus", type=Path, default=ROOT / "data/hive-workshop-models")
    execute.add_argument("--build-dir", type=Path, default=default_build)
    execute.add_argument("--results", type=Path, help="new directory; existing paths are refused")
    execute.add_argument("--dialects", nargs="+", choices=["engine", "hive"], default=["engine", "hive"])
    execute.add_argument("--timeout", type=float, default=30)
    execute.add_argument("--fail-on-review", action="store_true")
    args = parser.parse_args()
    try:
        if args.command == "build":
            build(args)
            return 0
        return run(args)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
