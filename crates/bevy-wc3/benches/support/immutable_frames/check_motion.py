"""Capture original moving controls through the maintained offscreen example."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("assets", type=Path, nargs="+")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--counts", default="1,128")
    parser.add_argument("--facing", default="0,2,3,4,5")
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    if args.output.exists():
        parser.error("output must be fresh")
    args.output = args.output.resolve()
    binary = args.binary.resolve()
    counts = {int(value) for value in args.counts.split(",")}
    modes = {int(value) for value in args.facing.split(",")}
    guards = {str(binary): digest(binary), str(Path(__file__).resolve()): digest(Path(__file__))}
    repo = Path(__file__).resolve().parents[5]
    author_paths = [repo / name for name in [
        "crates/wc3/tests/fixtures/pkb/generate_immutable_motion.py",
        "crates/wc3/tests/fixtures/pkb/generate_immutable.py",
        "crates/wc3/benches/fixtures/particle_runtime/generate.py",
    ]]
    guards.update({str(path): digest(path) for path in author_paths})
    jobs = []
    selected_cases = set()
    selected_names = set()
    for assets in args.assets:
        assets = assets.resolve()
        source = assets / "manifest.json"
        native = json.loads(source.read_text())
        guards[str(source)] = digest(source)
        for field, path in zip(["generator_sha256", "immutable_generator_sha256", "base_author_sha256"], author_paths):
            if native[field] != guards[str(path)]:
                raise ValueError("author provenance differs from generated controls")
        selected = {}
        for fixture in native["fixtures"]:
            if fixture["count"] not in counts or fixture["facing"] not in modes:
                continue
            case = Path(fixture["path"]).stem
            if case in selected or any(job["case"] == case for job in jobs):
                raise ValueError("duplicate case")
            selected[case] = fixture["count"]
            selected_cases.add((fixture["count"], fixture["facing"]))
            selected_names.add(case)
            for name, receipt in fixture["capture_files"].items():
                path = assets / case / name
                if digest(path) != receipt["sha256"] or path.stat().st_size != receipt["bytes"]:
                    raise ValueError("authored control changed")
                guards[str(path)] = receipt["sha256"]
        for job in native["capture_jobs"]:
            if job["case"] not in selected:
                continue
            name = job["case"] + "-" + job["route"]
            case_path = assets / job["case"]
            expected_options = ["--asset-root", str(case_path), "--times", "0.125,0.75,1.75,3.25",
                                "--fps", "64", "--size", "640x480", "--eye", "0,0,600",
                                "--target", "0,0,0", "--msaa", "1", "--no-default-light",
                                "--particle-stats", "--particle-execution", job["route"]]
            if job["argv"][0] != str(case_path / "model.mdl") or job["argv"][2:] != expected_options:
                raise ValueError("capture arguments do not identify the guarded authored case")
            argv = [str(binary), *job["argv"]]
            argv[2] = str(args.output.resolve() / name)
            jobs.append({"name": name, "case": job["case"], "children": selected[job["case"]],
                         "route": job["route"], "argv": argv})
    if selected_cases != {(count, mode) for count in counts for mode in modes}:
        raise ValueError("requested population/facing controls are incomplete")
    if {job["case"] for job in jobs} != selected_names:
        raise ValueError("requested fixture is missing its capture jobs")
    for case in selected_names:
        if sorted(job["route"] for job in jobs if job["case"] == case) != ["immutable", "interpreter"]:
            raise ValueError("control requires exactly one CPU/GPU pair")
    manifest = {"status": "planned", "hashes": guards, "jobs": jobs,
                "scope": "publication and omission checks; pixel comparisons and image inspection are separate"}
    if not args.execute:
        print(json.dumps(manifest, indent=2))
        return
    args.output.mkdir(parents=True)

    def save():
        (args.output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")

    def stable():
        if any(digest(Path(path)) != expected for path, expected in guards.items()):
            raise ValueError("frozen control changed")

    save()
    try:
        for job in jobs:
            stable()
            manifest.update(status="running", running=job["name"])
            save()
            try:
                result = subprocess.run(job["argv"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=300)
                stdout, stderr, code = result.stdout, result.stderr, result.returncode
                job["timed_out"] = False
            except subprocess.TimeoutExpired as error:
                stdout, stderr, code = error.stdout or b"", error.stderr or b"", None
                job["timed_out"] = True
            for kind, data in [("stdout", stdout), ("stderr", stderr)]:
                (args.output / (job["name"] + "." + kind)).write_bytes(data)
                job[kind + "_sha256"] = hashlib.sha256(data).hexdigest()
            job["returncode"] = code
            save()
            stable()
            if job["timed_out"] or code:
                raise RuntimeError("capture failed: " + job["name"])
            observations = [dict(re.findall(r"(\w+)=([^,]+)", line))
                            for line in stdout.decode().splitlines() if line.startswith("Programme ")]
            if len(observations) != 4:
                raise ValueError("missing per-capture publication observations")
            for row in observations:
                expected = job["children"] if job["route"] == "immutable" else 0
                if int(row["alive"]) != job["children"] + 1 or int(row["sprites"]) != job["children"] or int(row["immutable_sprites"]) != expected:
                    raise ValueError("wrong live or published population")
                if row["error"].strip() != "None" or any(float(row[key]) for key in ["ribbon_segments", "omitted_geometry", "omitted", "suppressed", "omitted_spatial_writes", "omitted_spatial_queries", "skipped_seconds"]):
                    raise ValueError("control fault, omission or unequal work")
            files = sorted(Path(job["argv"][2]).glob("frame-*.png"))
            if len(files) != 4:
                raise ValueError("incomplete images")
            job["observations"] = observations
            job["images"] = {str(path.relative_to(args.output)): digest(path) for path in files}
            save()
        manifest.update(status="captured_publication_checked_pixels_unreviewed")
        manifest.pop("running", None)
        save()
    except BaseException as error:
        manifest.update(status="failed", error=repr(error))
        save()
        raise


if __name__ == "__main__":
    main()
