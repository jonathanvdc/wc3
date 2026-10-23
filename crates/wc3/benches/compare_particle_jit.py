#!/usr/bin/env python3
"""Compare maintained interpreter/scalar-JIT/SIMD-JIT routes; --execute is required."""
import argparse
import hashlib
import itertools
import json
from pathlib import Path
import statistics
import subprocess

CASES = [(1, 128), (8, 128), (1, 1024), (8, 1024), (128, 128), (1024, 1024)]
ROUTES = ["interpreter", "scalar-jit", "simd-jit"]
METRICS = ["block_median", "block_p95", "frame_median", "frame_p95"]
COLUMNS = ["scalar_jit", "simd_batches", "simd_lanes", "rust_lanes"]
STEPS = 64


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def percentile(values, rank):
    return sorted(values)[(len(values) - 1) * rank // 100]


def expected_work(route, n):
    if route == "interpreter":
        return [0, 0, 0, n if n >= 16 else 0]
    if route == "scalar-jit" or n < 16:
        return [n + 1, 0, 0, 0]
    return [1, n // 16, n, 0]


def parse_output(stdout, route, children, capacity, instances, samples):
    rows = [json.loads(line) for line in stdout.decode().splitlines() if line.startswith("{")]
    if len(rows) != 1:
        raise ValueError("expected exactly one JSON result")
    row = rows[0]
    expected = dict(kind="result", route=route, children=children,
                    capacity_child_units=capacity, capacity_total=capacity + 1,
                    instances=instances, samples=samples, steps=STEPS, work_columns=COLUMNS)
    if any(row.get(key) != value for key, value in expected.items()):
        raise ValueError("result does not match requested workload/ledger layout")
    work = expected_work(route, children)
    for name, length, expected_delta in [
            ("work_deltas", STEPS, work),
            ("measured_work_deltas", samples * instances, [v * STEPS for v in work])]:
        ledger = row.get(name)
        if not isinstance(ledger, list) or len(ledger) != length or any(
                not isinstance(delta, list) or len(delta) != 4 or any(
                    type(v) is not int or v < 0 for v in delta) or delta != expected_delta
                for delta in ledger):
            raise ValueError("actual execution work disagrees with route: " + name)
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


def order(routes, repeat):
    shift = repeat % len(routes)
    selected = routes[shift:] + routes[:shift]
    return list(reversed(selected)) if (repeat // len(routes)) % 2 else selected


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("output", type=Path)
    default_repo = next((p for p in Path(__file__).resolve().parents
                         if (p / "crates/wc3/benches").is_dir()), Path.cwd())
    parser.add_argument("--repo", type=Path, default=default_repo)
    parser.add_argument("--routes", default=",".join(ROUTES),
                        help="comma-separated distinct routes; default compares all three")
    parser.add_argument("--repeats", type=int, default=5)
    parser.add_argument("--samples", type=int, default=31)
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    routes = args.routes.split(",")
    if len(routes) < 2 or len(set(routes)) != len(routes) or any(r not in ROUTES for r in routes):
        parser.error("--routes requires two or three distinct known routes")
    if args.repeats < 1 or args.samples < 5:
        parser.error("repeats must be positive; samples must be at least five")
    if args.output.exists():
        parser.error("output must be fresh")
    repo, binary = args.repo.resolve(), args.binary.resolve()
    files = [binary, Path(__file__).resolve(), repo / "Cargo.toml", repo / "Cargo.lock",
             repo / "crates/wc3/Cargo.toml", repo / "crates/wc3/examples/particle_jit_bench.rs",
             repo / "crates/wc3/benches/support/particle_runtime.rs"]
    def source_paths():
        paths = []
        for directory in ["crates/wc3/src/model/scene/popcorn", "crates/wc3/src/model/formats/pkb"]:
            paths += sorted((repo / directory).rglob("*.rs"))
        paths += sorted(p for p in (repo / "crates/wc3/benches/fixtures/particle_runtime").rglob("*")
                        if p.is_file())
        return paths

    frozen_sources = source_paths()
    files += frozen_sources
    hashes = {str(p): digest(p) for p in files}
    manifest = dict(status="planned", measurement=False, hashes=hashes, routes=routes,
                    cases=CASES, instances=[1, 8], repeats=args.repeats,
                    samples=args.samples, steps=STEPS, work_columns=COLUMNS,
                    comparisons=[], jobs=[], summaries=[])
    if not args.execute:
        print(json.dumps(manifest, indent=2))
        return
    args.output.mkdir(parents=True, exist_ok=False)

    def save():
        (args.output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")

    def stable():
        if source_paths() != frozen_sources:
            raise RuntimeError("immutable source/fixture inventory changed")
        for path, expected_hash in hashes.items():
            if digest(Path(path)) != expected_hash:
                raise RuntimeError("immutable input changed: " + path)

    save()
    try:
        for children, capacity in CASES:
            for instances in (1, 8):
                timing = {r: {m: [] for m in METRICS} for r in routes}
                cold = {r: {m: [] for m in ["compile_ns", "prepare_ns"]} for r in routes}
                comparisons = []
                for repeat in range(args.repeats):
                    rows = {}
                    for route in order(routes, repeat):
                        name = f"n{children}-c{capacity}-i{instances}-r{repeat}-{route}"
                        argv = [str(binary), route, str(children), str(capacity),
                                str(instances), str(args.samples)]
                        job = dict(name=name, argv=argv)
                        manifest["jobs"].append(job)
                        manifest["running"] = name
                        save()
                        stable()
                        result = subprocess.run(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                                timeout=600)
                        (args.output / (name + ".stdout")).write_bytes(result.stdout)
                        (args.output / (name + ".stderr")).write_bytes(result.stderr)
                        job.update(returncode=result.returncode,
                                   stdout_sha256=hashlib.sha256(result.stdout).hexdigest(),
                                   stderr_sha256=hashlib.sha256(result.stderr).hexdigest())
                        save()  # Preserve raw output before admission or hash checks.
                        stable()
                        if result.returncode or result.stderr:
                            raise RuntimeError(name + " failed or emitted stderr")
                        row = parse_output(result.stdout, route, children, capacity, instances,
                                           args.samples)
                        rows[route] = row
                        job["work_deltas"] = row["work_deltas"]
                        job["measured_work_deltas"] = row["measured_work_deltas"]
                        job["compile_ns"], job["prepare_ns"] = row["compile_ns"], row["prepare_ns"]
                        for metric in cold[route]:
                            cold[route][metric].append(row[metric])
                        for metric in METRICS:
                            timing[route][metric].append(row[metric + "_ns"])
                    for first, second in itertools.combinations(routes, 2):
                        a, b = rows[first], rows[second]
                        if a["structure"] != b["structure"]:
                            raise ValueError("structural trajectory changed: " + first + "/" + second)
                        pair = dict(children=children, capacity=capacity, instances=instances,
                                    repeat=repeat, denominator=first, numerator=second,
                                    numeric_bit_exact=a["numeric"] == b["numeric"])
                        for metric in METRICS:
                            pair[metric + "_ratio"] = b[metric + "_ns"] / a[metric + "_ns"]
                        comparisons.append(pair)
                        manifest["comparisons"].append(pair)
                    save()
                summary = dict(children=children, capacity=capacity, instances=instances,
                               all_numeric_bit_exact=all(p["numeric_bit_exact"] for p in comparisons),
                               pairs=[], strongest_by_metric={},
                               cold={r: {m: dict(raw_ns=values, median_ns=statistics.median(values))
                                         for m, values in cold[r].items()} for r in routes})
                for first, second in itertools.combinations(routes, 2):
                    subset = [p for p in comparisons if p["denominator"] == first and p["numerator"] == second]
                    pair = dict(denominator=first, numerator=second)
                    for metric in METRICS:
                        ratios = [p[metric + "_ratio"] for p in subset]
                        pair[metric] = dict(paired_ratios=ratios, median_ratio=statistics.median(ratios))
                    summary["pairs"].append(pair)
                for metric in METRICS:
                    medians = {r: statistics.median(timing[r][metric]) for r in routes}
                    best = min(routes, key=medians.get)
                    summary["strongest_by_metric"][metric] = dict(
                        route=best, median_raw_ns=medians,
                        ratio_to_strongest={r: medians[r] / medians[best] for r in routes})
                manifest["summaries"].append(summary)
                save()
        stable()
        manifest.update(status="complete_public_outputs_and_dispatch_verified", measurement=True)
        manifest.pop("running", None)
        save()
    except BaseException as error:
        manifest.update(status="failed", error=f"{type(error).__name__}: {error}")
        save()
        raise


if __name__ == "__main__":
    main()
