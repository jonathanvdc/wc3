#!/usr/bin/env python3
"""Compare the maintained particle_jit_bench routes; execution requires --execute."""
import argparse
import ast
import hashlib
import json
from pathlib import Path
import statistics
import subprocess

CASES = [(1, 128), (8, 128), (1, 1024), (8, 1024), (128, 128), (1024, 1024)]
STEPS = 64


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def percentile(values, numerator):
    return sorted(values)[(len(values) - 1) * numerator // 100]


def parse_output(stdout, route, children, capacity, instances, samples):
    lines = stdout.decode().splitlines()
    results = [json.loads(line) for line in lines if line.startswith("{")]
    deltas = [ast.literal_eval(line.removeprefix("invocation_deltas="))
              for line in lines if line.startswith("invocation_deltas=")]
    if len(results) != 1 or len(deltas) != 1:
        raise ValueError("expected one result and one actual invocation ledger")
    row = results[0]
    expected = dict(kind="result", route=route, children=children,
                    capacity_child_units=capacity, capacity_total=capacity + 1,
                    instances=instances, samples=samples, steps=STEPS)
    if any(row.get(key) != value for key, value in expected.items()):
        raise ValueError("result does not match requested workload")
    calls = deltas[0]
    expected_calls = children + 1 if route == "scalar-jit" else 0
    if not isinstance(calls, list) or len(calls) != STEPS or any(
            type(value) is not int or value != expected_calls for value in calls):
        raise ValueError("actual invocation ledger does not match requested route")
    for name, length in [("setup_ns", samples), ("blocks_ns", samples),
                         ("frames_ns", samples * STEPS)]:
        values = row[name]
        if len(values) != length or any(type(v) is not int or v <= 0 for v in values):
            raise ValueError("invalid raw duration samples: " + name)
    for prefix, raw in [("block", "blocks_ns"), ("frame", "frames_ns")]:
        for label, rank in [("median", 50), ("p95", 95)]:
            if row[f"{prefix}_{label}_ns"] != percentile(row[raw], rank):
                raise ValueError("reported quantile disagrees with raw samples")
    if len(row["structure"]) != STEPS + 1 or any(
            type(v) is not int or not 0 <= v < 2**64 for v in row["structure"]):
        raise ValueError("invalid structural trajectory")
    if len(row["numeric"]) != (STEPS + 1) * children * 14 or any(
            len(words) != 4 or any(type(v) is not int or not 0 <= v < 2**32
                                   for v in words) for words in row["numeric"]):
        raise ValueError("invalid canonical numeric trajectory")
    return row


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("output", type=Path)
    default_repo = next((parent for parent in Path(__file__).resolve().parents
                         if (parent / "crates/wc3/benches").is_dir()), Path.cwd())
    parser.add_argument("--repo", type=Path, default=default_repo)
    parser.add_argument("--repeats", type=int, default=5)
    parser.add_argument("--samples", type=int, default=31)
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    if args.repeats < 1 or args.samples < 5:
        parser.error("repeats must be positive and samples must be at least five")
    if args.output.exists():
        parser.error("output must be a fresh directory")
    repo = args.repo.resolve()
    binary = args.binary.resolve()
    files = [binary, Path(__file__).resolve(),
             repo / "crates/wc3/examples/particle_jit_bench.rs",
             repo / "crates/wc3/benches/support/particle_runtime.rs"]
    files += [repo / f"crates/wc3/benches/fixtures/particle_runtime/burst-{n}.pkb"
              for n in (1, 8, 128, 1024)]
    hashes = {str(path): digest(path) for path in files}
    manifest = dict(status="planned", hashes=hashes, cases=CASES, instances=[1, 8],
                    repeats=args.repeats, samples=args.samples, steps=STEPS,
                    comparisons=[], jobs=[], summaries=[])
    if not args.execute:
        print(json.dumps(manifest, indent=2))
        return
    args.output.mkdir(parents=True, exist_ok=False)

    def save():
        (args.output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")

    def stable():
        for path, expected_hash in hashes.items():
            if digest(Path(path)) != expected_hash:
                raise RuntimeError("immutable input changed: " + path)

    save()
    try:
        for children, capacity in CASES:
            for instances in (1, 8):
                pairs = []
                for repeat in range(args.repeats):
                    routes = ["interpreter", "scalar-jit"]
                    if repeat % 2:
                        routes.reverse()
                    rows = {}
                    for route in routes:
                        name = f"n{children}-c{capacity}-i{instances}-r{repeat}-{route}"
                        argv = [str(binary), route, str(children), str(capacity),
                                str(instances), str(args.samples)]
                        job = dict(name=name, argv=argv)
                        manifest["jobs"].append(job)
                        manifest["running"] = name
                        save()
                        stable()
                        result = subprocess.run(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                        (args.output / (name + ".stdout")).write_bytes(result.stdout)
                        (args.output / (name + ".stderr")).write_bytes(result.stderr)
                        job["returncode"] = result.returncode
                        save()  # Preserve logs before checking post-run hashes or admission.
                        stable()
                        if result.returncode or result.stderr:
                            raise RuntimeError(name + " failed or emitted stderr")
                        rows[route] = parse_output(result.stdout, route, children, capacity,
                                                   instances, args.samples)
                    base, jit = rows["interpreter"], rows["scalar-jit"]
                    if base["structure"] != jit["structure"]:
                        raise ValueError("structural trajectory changed")
                    pair = dict(children=children, capacity=capacity, instances=instances,
                                repeat=repeat, numeric_bit_exact=base["numeric"] == jit["numeric"])
                    for metric in ("block_median", "block_p95", "frame_median", "frame_p95"):
                        pair[metric + "_ratio"] = jit[metric + "_ns"] / base[metric + "_ns"]
                    pairs.append(pair)
                    manifest["comparisons"].append(pair)
                    save()
                summary = dict(children=children, capacity=capacity, instances=instances,
                               all_numeric_bit_exact=all(p["numeric_bit_exact"] for p in pairs))
                for metric in ("block_median", "block_p95", "frame_median", "frame_p95"):
                    ratios = [pair[metric + "_ratio"] for pair in pairs]
                    summary[metric] = dict(paired_ratios=ratios, median_ratio=statistics.median(ratios))
                manifest["summaries"].append(summary)
        stable()
        manifest["status"] = "complete_public_outputs_and_dispatch_verified"
        manifest.pop("running", None)
        save()
    except BaseException as error:
        manifest.update(status="failed", error=f"{type(error).__name__}: {error}")
        save()
        raise


if __name__ == "__main__":
    main()
