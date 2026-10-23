#!/usr/bin/env python3
"""Compare fourteen-field query methods; inspect the plan unless --execute is supplied."""
import argparse
import gzip
import hashlib
import importlib.util
import itertools
import json
from pathlib import Path
import statistics
import struct
import subprocess

ROUTES = ["interpreter", "scalar-jit", "simd-jit", "immutable-age"]
METHODS = ["individual", "bulk", "projection"]
CASES = [(128, 128, 1), (1024, 1024, 8)]
REPEATS = 5


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def numeric_digest(values):
    result = hashlib.sha256()
    for words in values:
        result.update(struct.pack("<4I", *words))
    return result.hexdigest()


def order(repeat):
    routes = ROUTES[repeat % len(ROUTES):] + ROUTES[:repeat % len(ROUTES)]
    if repeat % 2:
        routes.reverse()
    result = []
    for index, route in enumerate(routes):
        shift = (repeat + index) % len(METHODS)
        methods = METHODS[shift:] + METHODS[:shift]
        if repeat % 2:
            methods.reverse()
        result.extend((route, method) for method in methods)
    return result


def validate_queries(row, route, method, samples):
    costs = row["separate_costs"]
    if row.get("query_method") != method or costs.get("query_method") != method:
        raise ValueError("client did not honor the requested query method")
    if costs.get("query_allocation_deltas") != [[0, 0, 0]] * samples:
        raise ValueError("full-field queries allocated or omitted allocation proof")
    projected = route == "immutable-age" and method == "projection"
    for key in ["full_projection_counts", "position_projection_counts"]:
        counts = costs.get(key)
        if not isinstance(counts, list) or len(counts) != 3 or any(type(n) is not int or n < 0 for n in counts):
            raise ValueError("invalid projection preparation counts")
        if projected != all(n > 0 for n in counts):
            raise ValueError("unexpected projection preparation availability")
        if not projected and counts != [0, 0, 0]:
            raise ValueError("unavailable projection retained nonzero counts")
    if projected and costs["position_projection_counts"] != [1, 1, 1]:
        raise ValueError("stationary fixture position projection changed")
    for key in ["full_projection_prepare_ns", "position_projection_prepare_ns"]:
        value = costs.get(key)
        if type(value) is not int or value < 0 or (projected and value == 0) or (not projected and value != 0):
            raise ValueError("invalid cold projection preparation time")
    positions = costs.get("fixed_time_8_position_only_projection_sweeps_ns")
    if not isinstance(positions, list) or len(positions) != (samples if projected else 0) or any(type(n) is not int or n <= 0 for n in positions):
        raise ValueError("invalid separate position-only measurement")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[5])
    parser.add_argument("--samples", type=int, choices=[5, 7], default=5)
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    if args.output.exists():
        parser.error("output must be fresh; failed evidence is never overwritten")
    repo, binary = args.repo.resolve(), args.binary.resolve()
    bench = repo / "crates/wc3/benches"
    protocol_path = bench / "compare_particle_immutable.py"
    spec = importlib.util.spec_from_file_location("immutable_route_protocol", protocol_path)
    protocol = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(protocol)
    fixtures = repo / "crates/wc3/tests/fixtures/pkb"
    author = json.loads((fixtures / "immutable-manifest.json").read_text())
    if author["generator_sha256"] != digest(fixtures / "generate_immutable.py") or author["base_sha256"] != digest(repo / author["base"]):
        raise ValueError("fixture author differs from recorded provenance")
    if sorted(entry["count"] for entry in author["fixtures"]) != [1, 8, 128, 1024]:
        raise ValueError("unexpected fixture descriptor set")
    for entry in author["fixtures"]:
        path = fixtures / f"immutable-{entry['count']}.pkb"
        if entry["path"] != path.name or entry["sha256"] != digest(path) or entry["bytes"] != path.stat().st_size:
            raise ValueError("native fixture hash/length differs from its descriptor")

    def inventory():
        paths = [binary, Path(__file__).resolve(), Path(__file__).with_name("README.md").resolve(),
                 protocol_path, bench / "support/particle_immutable.rs",
                 repo / "crates/wc3/examples/particle_immutable_bench.rs",
                 repo / "Cargo.toml", repo / "Cargo.lock", repo / "crates/wc3/Cargo.toml",
                 fixtures / "immutable-manifest.json", fixtures / "generate_immutable.py"]
        paths += [fixtures / f"immutable-{n}.pkb" for n in [1, 8, 128, 1024]]
        for directory in ["crates/wc3/src/model/scene/popcorn", "crates/wc3/src/model/formats/pkb",
                          "crates/wc3/benches/fixtures/particle_runtime"]:
            paths.extend(path for path in (repo / directory).rglob("*") if path.is_file())
        return sorted(set(paths))

    paths = inventory()
    hashes = {str(path): digest(path) for path in paths}
    manifest = dict(status="planned", measurement=False, cases=CASES, repeats=REPEATS,
                    samples=args.samples, routes=ROUTES, query_methods=METHODS, hashes=hashes,
                    process_order=[order(repeat) for repeat in range(REPEATS)],
                    scope="client public fields, lifecycle, dispatch and allocation assertions; no private VM/RNG-state claim",
                    provenance="observed file/binary hashes, not proof that this binary was built from these sources; retain the build receipt",
                    numeric_policy="same-route bits identical across methods/repeats; cross-route comparison reuses the declared Graphics bound in compare_particle_immutable.py",
                    jobs=[], summaries=[])
    if not args.execute:
        print(json.dumps(manifest, indent=2))
        return
    args.output.mkdir(parents=True)

    def save():
        (args.output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")

    def stable():
        if inventory() != paths:
            raise RuntimeError("guarded source inventory changed")
        for path, expected in hashes.items():
            if digest(Path(path)) != expected:
                raise RuntimeError("guarded source/binary changed: " + path)

    def run(job):
        timed_out = False
        try:
            result = subprocess.run(job["argv"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=600)
            stdout, stderr, code = result.stdout, result.stderr, result.returncode
        except subprocess.TimeoutExpired as error:
            stdout, stderr, code = error.stdout or b"", error.stderr or b"", None
            timed_out = True
        raw = gzip.compress(stdout, mtime=0)
        (args.output / (job["name"] + ".stdout.gz")).write_bytes(raw)
        (args.output / (job["name"] + ".stderr")).write_bytes(stderr)
        job.update(returncode=code, timed_out=timed_out,
                   stdout_sha256=hashlib.sha256(stdout).hexdigest(),
                   stdout_gzip_sha256=hashlib.sha256(raw).hexdigest(),
                   stderr_sha256=hashlib.sha256(stderr).hexdigest())
        save()  # Logs survive failed protocol, oracle and subsequent hash checks.
        stable()
        if timed_out or code or stderr:
            raise RuntimeError(job["name"] + " timed out, failed, or emitted stderr")
        return stdout

    save()
    try:
        for children, capacity, instances in CASES:
            anchor, structure = None, None
            route_hashes = {}
            observations = []
            for repeat in range(REPEATS):
                for route, method in order(repeat):
                    name = f"n{children}-c{capacity}-i{instances}-r{repeat}-{route}-{method}"
                    job = dict(name=name, route=route, query_method=method, repeat=repeat,
                               children=children, capacity=capacity, instances=instances,
                               argv=[str(binary), route, str(children), str(capacity), str(instances),
                                     str(args.samples), "--query-method", method])
                    manifest["jobs"].append(job)
                    manifest["running"] = name
                    save()
                    stable()
                    row = protocol.parse_output(run(job), route, children, capacity, instances, args.samples)
                    validate_queries(row, route, method, args.samples)
                    if anchor is None:
                        if (route, method) != ("interpreter", "individual"):
                            raise ValueError("the reference must be Interpreter individual")
                        anchor, structure = row["numeric"], row["structure"]
                    if row["structure"] != structure:
                        raise ValueError("public structural trajectory changed")
                    protocol.compare_fields(anchor, row["numeric"], "interpreter", route)
                    numeric_hash = numeric_digest(row["numeric"])
                    if route in route_hashes and route_hashes[route] != numeric_hash:
                        raise ValueError("same-policy query method/repeat changed field bits")
                    route_hashes[route] = numeric_hash
                    costs = row["separate_costs"]
                    observation = dict(route=route, query_method=method, repeat=repeat,
                                       query_median_ns=statistics.median(costs["fixed_time_8_full_query_sweeps_ns"]),
                                       query_p95_ns=protocol.percentile(costs["fixed_time_8_full_query_sweeps_ns"], 95),
                                       frame_median_ns=row["frame_median_ns"], frame_p95_ns=row["frame_p95_ns"])
                    observations.append(observation)
                    job.update(numeric_sha256=numeric_hash, structure=structure,
                               work_deltas=row["work_deltas"], measured_work_deltas=row["measured_work_deltas"],
                               separate_costs=costs, compile_ns=row["compile_ns"], prepare_ns=row["prepare_ns"])
                    save()
                    del row
            summary = dict(children=children, capacity_child_units=capacity, capacity_total=capacity + 1,
                           instances=instances, numeric_sha256_by_route=route_hashes, observations=observations,
                           full_field_query_pairs=[], frame_advance_by_configuration=[])
            configurations = list(itertools.product(ROUTES, METHODS))
            query_medians = {}
            for route, method in configurations:
                selected = [row for row in observations if row["route"] == route and row["query_method"] == method]
                query_medians[(route, method)] = statistics.median(row["query_median_ns"] for row in selected)
                summary["frame_advance_by_configuration"].append(dict(
                    route=route, query_method=method,
                    frame_median_ns=[row["frame_median_ns"] for row in selected],
                    frame_p95_ns=[row["frame_p95_ns"] for row in selected]))
            for first, second in itertools.combinations(configurations, 2):
                comparison = dict(denominator=first, numerator=second)
                for metric in ["query_median_ns", "query_p95_ns"]:
                    ratios = []
                    for repeat in range(REPEATS):
                        values = {(row["route"], row["query_method"]): row[metric]
                                  for row in observations if row["repeat"] == repeat}
                        ratios.append(values[second] / values[first])
                    comparison[metric] = dict(paired_ratios=ratios, median_ratio=statistics.median(ratios))
                summary["full_field_query_pairs"].append(comparison)
            best = min(configurations, key=query_medians.get)
            summary["full_field_query_strongest"] = dict(configuration=best, configurations=[
                dict(route=route, query_method=method, median_ns=query_medians[(route, method)],
                     ratio_to_strongest=query_medians[(route, method)] / query_medians[best])
                for route, method in configurations])
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
