#!/usr/bin/env python3
"""Public eligibility/population audit. Never executes without --execute."""
import argparse
import collections
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import struct


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def key(row):
    return tuple(row[name] for name in ("path", "root", "evolution"))


def observation_sequence():
    result = [("start", 0, False), ("zero_before", 0, False)]
    result += [("advance", frame, True) for frame in range(1, 121)]
    result += [("zero_after", 120, False), ("stop", 120, False),
               ("stopped_zero_before", 0, False), ("stopped_advance", 1, True),
               ("stopped_zero_after", 1, False), ("restart", 0, False),
               ("restart_zero_before", 0, False)]
    result += [("restart_advance", frame, True) for frame in range(1, 121)]
    result += [("restart_zero_after", 120, False)]
    assert len(result) == 250
    return result


def summarize(path, expected_assets):
    statuses, layers, stages = {}, [], []
    prepared = collections.Counter()
    weighted = collections.defaultdict(collections.Counter)
    invocations, diagnostics = collections.Counter(), collections.Counter()
    previous, calls, endings = {}, collections.Counter(), {}
    files, compiled, instances = set(), set(), set()
    config = end = None
    sequence = observation_sequence()
    compile_results, instantiation_errors = set(), set()
    case_errors = collections.Counter()
    compiled_layers = {}
    with path.open() as stream:
        for line in stream:
            row = json.loads(line)
            kind = row["kind"]
            if end is not None:
                raise ValueError("rows after ending")
            if "root" in row:
                identity = key(row)
                if (row["path"] not in files or type(row["root"]) is not int
                        or row["root"] < 0 or row["evolution"] not in ("Physics", "Fixed", "Varying")):
                    raise ValueError("invalid root/evolution identity")
            if "seed" in row and row["seed"] not in (0, 42):
                raise ValueError("unexpected seed")
            if kind == "config":
                if config is not None:
                    raise ValueError("duplicate configuration")
                config = row
            elif kind == "file":
                if row["path"] in files:
                    raise ValueError("duplicate file")
                files.add(row["path"])
            elif kind == "compiled":
                if key(row) in compile_results:
                    raise ValueError("duplicate compile result")
                compile_results.add(key(row))
                compiled.add(key(row))
                compiled_layers[key(row)] = row["layers"]
            elif kind == "layer":
                if key(row) not in compiled or not 0 <= row["layer"] < compiled_layers[key(row)]:
                    raise ValueError("layer outside compiled programme")
                identity = key(row) + (row["layer"],)
                if identity in statuses:
                    raise ValueError("duplicate layer")
                statuses[identity] = row["immutable_expressions"]
                layers.append(row)
                for family in ("pure_packet", "immutable_expressions"):
                    verdict = row[family]
                    prepared[(family, verdict["status"], verdict.get("reason"))] += 1
            elif kind == "stage":
                stages.append(row)
            elif kind == "observation":
                instance = key(row) + (row["seed"],)
                if key(row) not in compiled or instance in endings or instance in instantiation_errors:
                    raise ValueError("observation outside open compiled case")
                ordinal = calls[instance]
                observed = (row["operation"], row["frame"], row["advancing"])
                if ordinal >= len(sequence) or observed != sequence[ordinal]:
                    raise ValueError(f"unexpected lifecycle observation: {instance} {ordinal} {observed}")
                instances.add(instance)
                calls[instance] += 1
                observed_layers = [layer["layer"] for layer in row["layers"]]
                if observed_layers != list(range(compiled_layers[key(row)])):
                    raise ValueError("observation layer inventory/order mismatch")
                if sum(layer["live"] for layer in row["layers"]) != row["active"]:
                    raise ValueError("layer live populations do not partition active count")
                for layer in row["layers"]:
                    if not 0 <= layer["current_immutable_samples"] <= layer["immutable_samples"] <= layer["live"]:
                        raise ValueError("invalid immutable observation counts")
                    verdict = statuses[key(row) + (layer["layer"],)]
                    group = (verdict["status"], verdict.get("reason"))
                    if row["advancing"]:
                        for metric in ("live", "immutable_samples", "current_immutable_samples"):
                            weighted[group][metric] += layer[metric]
                        weighted[group]["layer_observations"] += 1
                counters = row["instance_counters"]
                old = previous.get(instance, {})
                for name, value in counters.items():
                    delta = value - old.get(name, 0)
                    if delta < 0:
                        raise ValueError(f"unexpected counter reset: {instance} {name}")
                    invocations[name] += delta
                previous[instance] = counters
                if row["error"] is not None:
                    diagnostics["runtime_error_observations"] += 1
                    case_errors[instance] += 1
            elif kind == "case_end":
                instance = key(row) + (row["seed"],)
                if (instance in endings or row["observations"] != len(sequence)
                        or calls[instance] != len(sequence) or row["runtime_errors"] != case_errors[instance]):
                    raise ValueError("case completion mismatch")
                endings[instance] = row
            elif kind == "end":
                if end is not None:
                    raise ValueError("duplicate ending")
                end = row
            elif kind == "compile_error":
                if key(row) in compile_results:
                    raise ValueError("duplicate compile result")
                compile_results.add(key(row))
                diagnostics[kind] += 1
            elif kind == "instantiate_error":
                identity = key(row) + (row["seed"],)
                if key(row) not in compiled or identity in instantiation_errors or identity in instances:
                    raise ValueError("duplicate/invalid instantiation outcome")
                instantiation_errors.add(identity)
                diagnostics[kind] += 1
            else:
                diagnostics[kind] += 1
    if config is None or end is None or files != set(expected_assets):
        raise ValueError("incomplete corpus/configuration/ending")
    expected_config = dict(kind="config", version=1, files=58, capacity=128, seeds=[0, 42],
                           frames_per_epoch=120,
                           dt_bits=struct.unpack("<Q", struct.pack("<d", 1 / 60))[0],
                           execution="ImmutableAge", numeric_policy="Graphics", missing_payload="Zero")
    if config != expected_config or end["files"] != 58:
        raise ValueError("unexpected workload configuration")
    for file, root in {(file, root) for file, root, _ in compile_results}:
        if {evolution for f, r, evolution in compile_results if (f, r) == (file, root)} != {"Physics", "Fixed", "Varying"}:
            raise ValueError("missing root evolution outcome")
    expected_cases = {identity + (seed,) for identity in compiled for seed in (0, 42)}
    if set(endings) | instantiation_errors != expected_cases:
        raise ValueError("missing seed instantiation outcome")
    for identity, count in compiled_layers.items():
        if {layer for *owner, layer in statuses if tuple(owner) == identity} != set(range(count)):
            raise ValueError("incomplete prepared layer inventory")
    if len(compiled) != end["compiled"] or len(endings) != end["cases"] or instances != set(endings):
        raise ValueError("compiled/instance completion mismatch")
    failures = {"compile_error", "read_error", "document_error", "root_error", "no_effect_roots", "instantiate_error"}
    if sum(diagnostics[name] for name in failures) != end["preparation_errors"]:
        raise ValueError("unreported preparation failure")
    return {
        "end": end,
        "diagnostics": dict(diagnostics),
        "prepared_layer_counts": [dict(family=a, status=b, reason=c, layers=n)
                                  for (a, b, c), n in sorted(prepared.items(), key=lambda item: str(item[0]))],
        "advancing_observation_weights": [dict(status=a, reason=b, **values)
                                           for (a, b), values in sorted(weighted.items(), key=lambda item: str(item[0]))],
        "instance_only_actual_counter_deltas": dict(invocations),
        "layers": layers,
        "prepared_stages": stages,
        "interpretation": "Weights are observed population/sample exposures, never execution counts. Stage routes are prepared; native entries are instance-wide and never assigned to layers. No GPU rendering is measured.",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--reference-manifest", type=Path)
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    repo, binary = args.repo.resolve(), args.binary.resolve()
    source = repo / "crates/wc3/examples/particle_specialization_audit.rs"
    runner = Path(__file__).resolve()

    def assets():
        paths = sorted((repo / "data/dependencies/files").rglob("*.pkb"))
        if len(paths) != 58 or any(p.is_symlink() for p in paths):
            raise ValueError("expected 58 ordinary native PKBs")
        return {str(p.relative_to(repo)): sha(p) for p in paths}

    def source_inventory():
        paths = [source, runner, runner.with_name("README.md"),
                 repo / "Cargo.toml", repo / "Cargo.lock", repo / "crates/wc3/Cargo.toml"]
        for directory in ("crates/wc3/src/model/scene/popcorn", "crates/wc3/src/model/formats/pkb"):
            paths.extend(path for path in (repo / directory).rglob("*") if path.is_file())
        return sorted(set(paths))

    sources = source_inventory()
    source_hashes = {str(path): sha(path) for path in sources}
    pins = assets()
    if args.reference_manifest and json.loads(args.reference_manifest.read_text())["assets"] != pins:
        raise ValueError("asset hashes differ from prior corpus manifest")
    manifest = {"status": "plan", "assets": pins, "source_sha256": sha(source),
                "runner_sha256": sha(runner), "binary": str(binary), "binary_sha256": sha(binary),
                "argv": [str(binary), str(repo)], "guarded_sources": source_hashes,
                "provenance": "Observed source/binary hashes; retain the build receipt linking them."}
    if args.reference_manifest:
        manifest["reference_manifest"] = {"path": str(args.reference_manifest.resolve()), "sha256": sha(args.reference_manifest)}
    if not args.execute:
        print(json.dumps(manifest, indent=2, sort_keys=True))
        return 0
    args.output.mkdir(parents=True, exist_ok=False)

    def save():
        (args.output / "manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")

    def stable():
        if source_inventory() != sources or any(sha(Path(path)) != expected for path, expected in source_hashes.items()):
            raise ValueError("guarded core source inventory or Cargo inputs changed")
        if assets() != pins or sha(source) != manifest["source_sha256"] or sha(runner) != manifest["runner_sha256"] or sha(binary) != manifest["binary_sha256"]:
            raise ValueError("guarded corpus/source/runner/binary changed")

    save()
    try:
        stable()
        with (args.output / "coverage.jsonl").open("wb") as out, (args.output / "stderr.txt").open("wb") as err:
            result = subprocess.run(manifest["argv"], stdout=out, stderr=err)
        manifest["returncode"] = result.returncode
        stable()
        if result.returncode or (args.output / "stderr.txt").stat().st_size:
            raise ValueError("client failed or produced unexpected stderr; logs retained")
        summary = summarize(args.output / "coverage.jsonl", pins)
        (args.output / "summary.json").write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
        manifest["stdout_sha256"] = sha(args.output / "coverage.jsonl")
        manifest["summary_sha256"] = sha(args.output / "summary.json")
        manifest["status"] = "completed_with_diagnostics" if any(summary["diagnostics"].values()) else "completed"
        save()
        return 0
    except BaseException as error:
        manifest["status"], manifest["error"] = "failed", str(error)
        save()
        print(str(error), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
