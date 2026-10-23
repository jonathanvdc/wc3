"""Guarded paired advancing capture measurements using an optimized capture binary."""
import argparse
import csv
import hashlib
import io
import json
import math
from decimal import Decimal
from pathlib import Path
import statistics
import subprocess

FRAMES = 64
ROUTES = ["simd-jit", "immutable"]
WORK = ["scalar_jit_invocations", "rust_packet_committed_lanes", "simd_attempted_batches",
        "simd_committed_lanes", "immutable_age_invocations", "immutable_age_fallbacks"]
BUDGETS = ["omitted_geometry", "omitted", "suppressed", "skipped_seconds"]
CATEGORIES = ["birth_records", "sampled_records", "resident_records", "natural_indices", "view_orders", "uniforms"]
TRAFFIC = ["queue_bytes", "queue_writes", "initial_bytes", "initializations"]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def encoded_hash(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True).encode()).hexdigest()


def rows(path):
    return list(csv.DictReader(io.StringIO(path.read_text())))


def timing(values):
    ordered = sorted(values)
    return {"median_ms": ordered[len(ordered) // 2],
            "p95_ms": ordered[len(ordered) - len(ordered) // 20 - 1]}


def admit(directory, features, route, children):
    samples = rows(directory / "advance-timings.csv")
    if len(samples) != FRAMES or [int(row["frame"]) for row in samples] != list(range(FRAMES)):
        raise ValueError("wrong advancing sample count/order")
    for row in samples:
        if any(not math.isfinite(float(row[key + "_ms"])) or float(row[key + "_ms"]) < 0 for key in ["wall", "step", "wait"]) or float(row["wall_ms"]) <= 0:
            raise ValueError("invalid raw synchronized timing")
        expected = {"programme_particles": features * (children + 1),
                    "programme_sprites": features * children,
                    "immutable_sprites": features * children if route == "immutable" else 0}
        if any(int(row[key]) != value for key, value in expected.items()):
            raise ValueError("unexpected published frame populations")
    programmes = rows(directory / "advance-programmes.csv")
    grouped = {}
    for row in programmes:
        grouped.setdefault(int(row["frame"]), []).append(row)
        if int(row["alive"]) != children + 1 or int(row["sprites"]) != children:
            raise ValueError("unexpected per-feature programme population")
        if int(row["ribbon_segments"]) or any(float(row[key]) != 0 for key in BUDGETS):
            raise ValueError("unmatched ribbon/budget work")
        if not row["simulation_seconds"]:
            raise ValueError("instance clock unavailable")
        if int(row["immutable_sprites"]) != (children if route == "immutable" else 0):
            raise ValueError("requested mode did not publish all/none through immutable transport")
    if set(grouped) != set(range(-1, FRAMES)) or any(len(v) != features for v in grouped.values()):
        raise ValueError("missing baseline or per-feature observations")
    actual_work = []
    for frame in range(FRAMES):
        previous = {row["entity"]: row for row in grouped[frame - 1]}
        current = {row["entity"]: row for row in grouped[frame]}
        if len(previous) != features or current.keys() != previous.keys():
            raise ValueError("feature identity/replacement changed during stable interval")
        total = [0] * len(WORK)
        for entity, row in current.items():
            if Decimal(row["simulation_seconds"]) <= Decimal(previous[entity]["simulation_seconds"]):
                raise ValueError("actual core clock did not advance")
            delta = [int(row[key]) - int(previous[entity][key]) for key in WORK]
            if any(value < 0 for value in delta):
                raise ValueError("cumulative route counter reset")
            if route == "immutable":
                if delta[4] <= 0 or any(delta[i] for i in [1, 2, 3, 5]) or int(row[WORK[5]]):
                    raise ValueError("missing immutable authorizations or actual fallback/substituted packet work")
            elif delta[3] <= 0 or delta[2] <= 0 or any(delta[i] for i in [1, 4, 5]):
                raise ValueError("missing actual SIMD entries/lanes or substituted Rust/immutable work")
            total = [a + b for a, b in zip(total, delta)]
        actual_work.append(total)
    # Preserve full observable equality without requiring process-local Entity IDs.
    fields = ["simulation_seconds", "alive", "sprites", "ribbon_segments", *BUDGETS]
    observable = {frame: sorted(tuple(row[key] for key in fields) for row in data)
                  for frame, data in grouped.items()}
    before, after = {}, {}
    for row in rows(directory / "advance-uploads.csv"):
        target = before if row["observation"] == "before" else after if row["observation"] == "after" else None
        if target is None or row["category"] in target:
            raise ValueError("unexpected/duplicate upload snapshot")
        target[row["category"]] = {key: int(row[key]) for key in TRAFFIC}
    if set(before) != set(CATEGORIES) or set(after) != set(CATEGORIES):
        raise ValueError("incomplete upload categories")
    upload_delta = {category: {key: after[category][key] - before[category][key]
                               for key in TRAFFIC} for category in CATEGORIES}
    if any(value < 0 for traffic in upload_delta.values() for value in traffic.values()):
        raise ValueError("upload counters reset")
    if route == "immutable" and any(upload_delta["birth_records"].values()):
        raise ValueError("stable immutable birth rows still uploaded")
    if route == "simd-jit" and upload_delta["sampled_records"]["queue_bytes"] <= 0:
        raise ValueError("changing CPU records did not execute an upload")
    if route == "immutable" and upload_delta["uniforms"]["queue_bytes"] <= 0:
        raise ValueError("advancing shader time did not update its uniform")
    summary = {component: timing([float(row[component + "_ms"]) for row in samples])
               for component in ["wall", "step", "wait"]}
    return {"timing": summary, "observations_sha256": encoded_hash(observable),
            "frame_clocks": [[row[key] for key in ["requested_seconds", "engine_seconds", "delta_seconds"]]
                             for row in samples],
            "work_columns": WORK, "actual_work_per_external_frame": actual_work,
            "static_before_uploads": before, "advancing_upload_deltas": upload_delta,
            "clock_precision": "actual published core/Bevy clocks as9-decimal capture CSV, not rawf64 bits"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("assets", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--build-label", required=True)
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    binary, assets, output = args.binary.resolve(), args.assets.resolve(), args.output.resolve()
    if output.exists():
        parser.error("output must be fresh")
    native = json.loads((assets / "manifest.json").read_text())
    guards = {str(binary): digest(binary), str(Path(__file__).resolve()): digest(Path(__file__).resolve()),
              str(assets / "manifest.json"): digest(assets / "manifest.json"), **native["sources"]}
    for name, value in native["files"].items():
        path = assets / name
        if digest(path) != value["sha256"] or path.stat().st_size != value["bytes"]:
            raise ValueError("authored closure changed")
        guards[str(path)] = value["sha256"]
    manifest = {"status": "planned", "measurement": False, "build_label": args.build_label,
                "hashes": guards, "repeats": 5, "frames": FRAMES, "jobs": [], "pairs": [], "summaries": [],
                "boundaries": "clock includes natural simulation/render/submission/completion; work reads/CSV I/O outside; same SpinPoll wait",
                "substeps": "actual measured route deltas retained, no64*N invocation assumption"}
    if not args.execute:
        print(json.dumps(manifest, indent=2))
        return
    output.mkdir(parents=True)

    def stable():
        for name, expected in guards.items():
            if digest(Path(name)) != expected:
                raise RuntimeError(f"frozen input changed: {name}")

    def save():
        (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")

    save()
    try:
        for features in native["features"]:
            ratios = {component: {metric: [] for metric in ["median_ms", "p95_ms"]}
                      for component in ["wall", "step", "wait"]}
            for repeat in range(5):
                pair = {}
                order = ROUTES if repeat % 2 == 0 else list(reversed(ROUTES))
                for route in order:
                    stable()
                    name = f"features{features}-r{repeat}-{route}"
                    directory = output / name
                    argv = [str(binary), str(assets / f"instances-{features}.mdl"), str(directory),
                            "--asset-root", str(assets), *native["options"], "--particle-execution", route]
                    job = {"name": name, "features": features, "repeat": repeat, "route": route, "argv": argv}
                    manifest["jobs"].append(job)
                    manifest["status"] = "running"
                    manifest["running"] = name
                    save()
                    result = subprocess.run(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=900)
                    (output / (name + ".stdout")).write_bytes(result.stdout)
                    (output / (name + ".stderr")).write_bytes(result.stderr)
                    job["returncode"] = result.returncode
                    job["stdout_sha256"] = hashlib.sha256(result.stdout).hexdigest()
                    job["stderr_sha256"] = hashlib.sha256(result.stderr).hexdigest()
                    save()
                    stable()
                    if result.returncode:
                        raise RuntimeError(f"capture failed: {name}")
                    proof = admit(directory, features, route, native["children_per_feature"])
                    job["admission"] = proof
                    job["files"] = {str(p.relative_to(output)): digest(p) for p in sorted(directory.iterdir()) if p.is_file()}
                    pair[route] = proof
                    save()
                a, b = pair["simd-jit"], pair["immutable"]
                if a["observations_sha256"] != b["observations_sha256"] or a["frame_clocks"] != b["frame_clocks"]:
                    raise ValueError("per-frame core/population/budget/input-clock mismatch")
                comparison = {"features": features, "repeat": repeat, "numeric_observations_match": True,
                              "ratios_immutable_over_simd": {}}
                for component in ratios:
                    comparison["ratios_immutable_over_simd"][component] = {}
                    for metric in ratios[component]:
                        denominator = a["timing"][component][metric]
                        value = b["timing"][component][metric] / denominator if denominator else None
                        comparison["ratios_immutable_over_simd"][component][metric] = value
                        ratios[component][metric].append(value)
                manifest["pairs"].append(comparison)
                save()
            manifest["summaries"].append({"features": features,
                "paired_ratios": ratios,
                "median_paired_ratio_immutable_over_simd": {
                    component: {metric: statistics.median(values) if all(v is not None for v in values) else None
                                for metric, values in metrics.items()}
                    for component, metrics in ratios.items()}})
        stable()
        manifest["status"] = "complete_actual_routes_clocks_populations_uploads_admitted_no_speedup_assumed"
        manifest["measurement"] = True
        manifest.pop("running", None)
        save()
    except BaseException as error:
        manifest["status"] = "failed"
        manifest["error"] = repr(error)
        save()
        raise


if __name__ == "__main__":
    main()
