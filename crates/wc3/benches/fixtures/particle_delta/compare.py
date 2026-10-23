"""Guarded, serial four-route Delta benchmark; explicit --execute is required.

Five rotated/reversed rounds of five workloads and four routes: 100 processes.
Each client performs five samples of 64 advances after its full source admission.
"""
import argparse
import hashlib
import json
import statistics
import subprocess
import time
from pathlib import Path

ROUTES = ["interpreter", "scalar-jit", "simd-jit", "immutable-age"]
CASES = [("density", n) for n in [1, 128, 1024, 8192]] + [("routed", 1)]
ROUNDS, SAMPLES, STEPS, WARM = 5, 5, 64, 32
DT = 1 / 64
ADMITTED_PACKET_LANES = 16  # Measured default, asserted through actual counters.
ROUTED_LIVE_CHILDREN = 16  # Lifetime / DT; independent of packet width.
ROUTED_CAPACITY = 32
WORK_COLUMNS = ["scalar_jit", "simd_batches", "simd_lanes", "rust_lanes",
                "immutable_age", "immutable_fallbacks"]


def digest(path):
    blob = path.read_bytes()
    return {"sha256": hashlib.sha256(blob).hexdigest(), "bytes": len(blob)}


def inventory(repo, binary):
    paths = {binary, Path(__file__).resolve(), repo / "Cargo.toml", repo / "Cargo.lock"}
    paths.update(repo.glob("crates/*/Cargo.toml"))
    for directory in ["crates/wc3/src", "crates/wc3-derive/src"]:
        paths.update((repo / directory).rglob("*.rs"))
    for relative in [
        "crates/wc3/examples/particle_delta_bench.rs",
        "crates/wc3/benches/support/particle_delta.rs",
        "crates/wc3/benches/support/benchmark_alloc.rs",
        "crates/wc3/benches/fixtures/particle_runtime/generate.py",
        "crates/wc3/benches/fixtures/particle_delta/generate.py",
        "crates/wc3/tests/fixtures/pkb/generate_delta_affine.py",
        "crates/wc3/tests/fixtures/pkb/delta-affine.pkb",
    ]:
        paths.add(repo / relative)
    paths.update(repo / f"crates/wc3/benches/fixtures/particle_delta/density-{n}.pkb"
                 for kind, n in CASES if kind == "density")
    return {str(path): digest(path) for path in sorted(paths)}


def write_json(path, value):
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, indent=2) + "\n")
    temporary.replace(path)


def quantile(values, percentile):
    values = sorted(values)
    return values[(len(values) - 1) * percentile // 100]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def parse(stdout):
    lines = stdout.decode("utf-8").splitlines()
    routes = [line for line in lines if line.startswith("route=")]
    require(len(routes) == 1, "expected exactly one prepared-route Debug line")
    data = [json.loads(line) for line in lines if not line.startswith("route=")]
    require(len(data) == 2, "expected exactly two JSON records")
    records = {row["kind"]: row for row in data}
    require(set(records) == {"contract", "result"}, "unexpected JSON record kinds")
    return routes[0], records["contract"], records["result"]


def validate(contract, result, route, kind, count):
    require(contract["debug_assertions"] is False, "debug builds are not admitted for timing")
    require(contract["route"] == route and contract["workload"] == kind, "wrong workload")
    require(contract["requested_children_or_occurrences"] == count, "wrong population input")
    for name, expected in [("dt", DT), ("warmup", WARM), ("steps", STEPS), ("samples", SAMPLES),
                           ("abs", 0.000002), ("rel", 0.000002)]:
        # Rust formats the f32 tolerance as its shortest roundtripping decimal.
        require(contract[name] == expected, f"unexpected contract {name}")
    require(contract["work_columns"] == WORK_COLUMNS, "counter schema changed")
    routed = kind == "routed"
    live = ROUTED_LIVE_CHILDREN if routed else count
    require(contract["capacity_total"] == (ROUTED_CAPACITY if routed else count + 1), "capacity changed")
    ledger = contract["ledger"]
    require(len(ledger) == STEPS, "missing per-frame admission rows")
    total = [0] * len(WORK_COLUMNS)
    for index, row in enumerate(ledger):
        require(len(row) == 5, "invalid ledger row")
        clock, active, spawned, retired, work = row
        require(clock == (WARM + index + 1) * DT, "simulation clock mismatch")
        require(active == live + 1, "population mismatch")
        expected_spawned = WARM + index + 2 if routed else count + 1
        require(spawned == expected_spawned and retired == spawned - active,
                "spawn/death ledger mismatch")
        require(len(work) == 6 and all(type(x) is int and x >= 0 for x in work),
                "invalid actual execution counters")
        require(work[5] == 0, "immutable fallback is not admitted")
        for i, value in enumerate(work):
            total[i] += value
        if route == "interpreter":
            require(work == [0, 0, 0, live if not routed and count >= ADMITTED_PACKET_LANES else 0, 0, 0],
                    "wrong Rust execution route")
        elif route == "scalar-jit":
            require(work == [live + (2 if routed else 1), 0, 0, 0, 0, 0],
                    "wrong scalar execution route")
        elif route == "simd-jit":
            expected = [1, live // ADMITTED_PACKET_LANES, live, 0, 0, 0] if not routed and count >= ADMITTED_PACKET_LANES else [live + (2 if routed else 1), 0, 0, 0, 0, 0]
            require(work == expected, "wrong actual SIMD/tail execution route")
        else:
            require(work == [2 if routed else 1, 0, 0, 0, live, 0],
                    "wrong immutable execution route")
    require(contract["measured_work"] == [total] * SAMPLES,
            "measured work differs from admitted actual work")
    require(len(result["frames_ns"]) == STEPS * SAMPLES, "missing raw frame times")
    require(all(type(x) is int and x > 0 for x in result["frames_ns"]), "invalid frame duration")
    for percentile, key in [(50, "frame_median_ns"), (95, "frame_p95_ns")]:
        require(result[key] == quantile(result["frames_ns"], percentile), "quantile changed")
    for key in ["instantiate_ns", "start_and_first_birth_tick_ns", "setup_ns",
                "eight_full_query_sweeps_ns", "eight_selected_query_sweeps_ns"]:
        require(len(result[key]) == SAMPLES and all(type(x) is int and x > 0 for x in result[key]),
                f"invalid separate cost: {key}")
    for key in ["hot_allocation_deltas", "query_allocation_deltas"]:
        require(result[key] == [[0, 0, 0]] * SAMPLES, f"nonzero allocation work: {key}")
    require(result["selected_query_projected"] == (route == "immutable-age"),
            "wrong selected-field query implementation")
    observed = ("native SIMD" if total[2] else "checked Rust packet" if total[3]
                else "immutable Delta DAG" if total[4] else "scalar native" if total[0]
                else "checked scalar interpreter")
    return total, observed


def paired(records):
    summaries = []
    for kind, count in CASES:
        runs = {(row["round"], row["route"]): row for row in records
                if row["workload"] == kind and row["count"] == count}
        cpu = min(ROUTES[:3], key=lambda route: statistics.median(
            runs[round_, route]["result"]["frame_median_ns"] for round_ in range(ROUNDS)))
        comparisons = [(route, "interpreter") for route in ROUTES[1:]]
        comparisons.append(("immutable-age", cpu))
        rows = []
        for numerator, denominator in comparisons:
            metrics = {}
            for name, accessor in [
                ("advance_median", lambda x: x["frame_median_ns"]),
                ("advance_p95", lambda x: x["frame_p95_ns"]),
                ("full_queries_median", lambda x: statistics.median(x["eight_full_query_sweeps_ns"])),
                ("selected_queries_median", lambda x: statistics.median(x["eight_selected_query_sweeps_ns"])),
            ]:
                ratios = [accessor(runs[r, numerator]["result"]) /
                          accessor(runs[r, denominator]["result"]) for r in range(ROUNDS)]
                metrics[name] = {"paired_ratios": ratios, "median_paired_ratio": statistics.median(ratios),
                                 "individual_regressions": sum(value > 1 for value in ratios)}
            rows.append({"numerator": numerator, "denominator": denominator, "metrics": metrics})
        summaries.append({"workload": kind, "count": count,
                          "fastest_cpu_requested_route": cpu,
                          "fastest_cpu_actual_routes": sorted({runs[r, cpu]["observed"] for r in range(ROUNDS)}),
                          "selection": "fixed route chosen post-hoc by median of five per-process advance medians",
                          "comparisons": rows})
    return summaries


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--timeout", type=float, default=600)
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    repo, binary, output = args.repo.resolve(), args.binary.resolve(), args.output.resolve()
    require(args.timeout > 0, "timeout must be positive")
    require(not output.exists(), "output directory must be fresh")
    inputs = inventory(repo, binary)
    plan = []
    for round_ in range(ROUNDS):
        for case_index, (kind, count) in enumerate(CASES):
            routes = ROUTES[:] if round_ % 2 == 0 else ROUTES[::-1]
            rotation = (round_ + case_index) % len(routes)
            routes = routes[rotation:] + routes[:rotation]
            for route in routes:
                job = f"round{round_}-{kind}{count}-{route}"
                plan.append({"id": job, "round": round_, "route": route,
                             "workload": kind, "count": count,
                             "argv": [str(binary), route, kind, str(count), str(SAMPLES)]})
    manifest = {"status": "planned_not_executed", "input_hashes": inputs,
                "contract": {"rounds": ROUNDS, "samples_per_process": SAMPLES,
                             "steps": STEPS, "warmup": WARM, "dt": DT,
                             "graphics_abs": 0.000002, "graphics_rel": 0.000002},
                "plan": plan, "runs": [], "failure": None}
    if not args.execute:
        print(json.dumps(manifest, indent=2))
        return
    output.mkdir(parents=True)
    target = output / "manifest.json"
    manifest["status"] = "running"
    write_json(target, manifest)
    semantic = {}
    repeat_outputs = {}
    try:
        for job in plan:
            manifest["running_job"] = job["id"]
            manifest.pop("last_process", None)
            write_json(target, manifest)
            require(inventory(repo, binary) == inputs, "inputs changed before job")
            directory = output / job["id"]
            directory.mkdir()
            manifest["running_job"] = job["id"]
            write_json(target, manifest)
            begin = time.monotonic()
            try:
                process = subprocess.run(job["argv"], stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                         timeout=args.timeout, check=False, cwd=repo)
            except subprocess.TimeoutExpired as error:
                (directory / "stdout").write_bytes(error.stdout or b"")
                (directory / "stderr").write_bytes(error.stderr or b"")
                raise RuntimeError(f"timeout: {job['id']}") from error
            (directory / "stdout").write_bytes(process.stdout)
            (directory / "stderr").write_bytes(process.stderr)
            manifest["last_process"] = {
                "id": job["id"], "returncode": process.returncode,
                "logs": {name: {"path": str(directory / name), **digest(directory / name)}
                         for name in ["stdout", "stderr"]},
            }
            write_json(target, manifest)
            require(inventory(repo, binary) == inputs, "inputs changed during job")
            require(process.returncode == 0 and not process.stderr,
                    f"failed process or nonempty stderr: {job['id']}")
            debug, contract, result = parse(process.stdout)
            total, observed = validate(contract, result, job["route"], job["workload"], job["count"])
            case = job["workload"], job["count"]
            common = {"semantic_ledger": [row[:4] for row in contract["ledger"]],
                      "source_hash": contract["source_hash"], "capacity": contract["capacity_total"]}
            require(semantic.setdefault(case, common) == common, "cross-route semantic ledger/source mismatch")
            key = (*case, job["route"])
            actual_hash = contract["final_candidate_hash"]
            require(repeat_outputs.setdefault(key, actual_hash) == actual_hash,
                    "same-route repeated candidate outputs changed")
            record = {**job, "process_seconds": time.monotonic() - begin, "observed": observed,
                      "prepared_debug": debug, "actual_work_sum": total,
                      "contract": contract, "result": result,
                      "logs": {name: {"path": str(directory / name), **digest(directory / name)}
                               for name in ["stdout", "stderr"]}}
            manifest["runs"].append(record)
            write_json(target, manifest)
        require(inventory(repo, binary) == inputs, "inputs changed after final job")
        manifest["status"] = "all_admissions_completed"
        manifest.pop("running_job", None)
        manifest.pop("last_process", None)
        manifest["paired_summary"] = paired(manifest["runs"])
        manifest["interpretation"] = [
            "Client compares every canonical field/identity against ordinary source before clocks.",
            "Ordinary floating fields use the declared Graphics bound; canonical lifetime/root/Boolean/inactive words stay exact.",
            "Cross-route source hash and semantic ledgers are exact; selected-route numeric hashes may differ within the client-checked bound.",
            "Actual route counters, not requested route names, identify vector execution; routed and density1 SIMD requests use scalar tails.",
            "Cold preparation and public query costs are separate; ratios use paired process quantiles, not ratios of pooled run medians.",
            "Fastest CPU comparator is selected from measured admitted CPU routes post-hoc and held fixed across its five pairs.",
            "Rust global allocator counters exclude foreign C/driver allocations.",
        ]
        write_json(target, manifest)
    except Exception as error:
        manifest["status"] = "failed"
        manifest["failure"] = {"job": manifest.get("running_job"), "error": str(error)}
        try:
            after = inventory(repo, binary)
            manifest["failure"]["input_hashes_match_after_failure"] = after == inputs
            manifest["failure"]["changed_inputs"] = [key for key in set(inputs) | set(after)
                                                       if inputs.get(key) != after.get(key)]
        except OSError as inventory_error:
            manifest["failure"]["input_inventory_error"] = str(inventory_error)
        write_json(target, manifest)
        raise


if __name__ == "__main__":
    main()
