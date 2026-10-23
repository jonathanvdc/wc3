"""Author original steady Delta-affine benchmarks; never alter existing fixtures.

Run from the repository root:
  python3 crates/wc3/benches/fixtures/particle_delta/generate.py
Encoding and metadata checks do not establish strict import or execution.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=Path(__file__).resolve().parent)
    args = parser.parse_args()
    base = Path(__file__).resolve().parent.parent / "particle_runtime/generate.py"
    spec = importlib.util.spec_from_file_location("original_particle_author", base)
    author = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(author)
    original_fields = author.Document.fields
    original_script = author.Document.script

    def fields(self, rows):
        if any(row[0] == "coefficient" for row in rows):
            replacements = {
                "self.invLife": [.125],
                "basePosition": [1., -.5, .25],
                "Position": [.125, .25, -.5],
                "Rotation": [3.],
                "coefficient": [2.],
            }
            rows = [(name, ty, [author.F(v) for v in replacements[name]]
                     if name in replacements else words) for name, ty, words in rows]
        return original_fields(self, rows)

    def script(self, stage, code, constants, externals, functions, slots, names):
        if stage == 0 and names and names[0] == "store self.lifeRatio" and len(names) == 14:
            # Base author builds its init constant pool from the original rows,
            # separately from Document.fields. Keep both authored sources equal.
            constants = [list(words) for words in constants]
            for index, values in {1: [.125], 2: [1., -.5, .25],
                                  7: [2.], 8: [.125, .25, -.5], 10: [3.]}.items():
                constants[index] = [author.F(v) for v in values]
        elif stage == 3 and names and names[0] == "load scene.time":
            externals = list(externals) + [self.external("scene.dt", 31, 1, 2)]
            delta = len(externals) - 1
            code = b"".join([
                author.load(33, 0, 9),
                author.load(33, 1, 3),
                author.load(31, 2, delta),
                author.binary(2, 33, 1, author.reg(33, 1), author.reg(31, 2)),
                author.binary(0, 33, 0, author.reg(33, 0), author.reg(33, 1)),
                author.store(author.reg(33, 0), 9),
                author.load(31, 3, 11),
                author.load(31, 4, 8),
                author.binary(2, 31, 4, author.reg(31, 4), author.reg(31, 2)),
                author.binary(0, 31, 3, author.reg(31, 3), author.reg(31, 4)),
                author.store(author.reg(31, 3), 11),
            ])
            constants, functions, slots = [], [], 5
            names = ["load Position", "load constant velocity", "load scene.dt",
                     "velocity*Delta", "Position+increment", "store Position",
                     "load Rotation", "load constant angular rate", "rate*Delta",
                     "Rotation+increment", "store Rotation"]
        return original_script(self, stage, code, constants, externals, functions, slots, names)

    author.Document.fields = fields
    author.Document.script = script
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    rows = []
    for count in [1, 128, 1024, 8192]:
        blob, metadata = author.build(count)
        target = output / f"density-{count}.pkb"
        if target.exists():
            raise SystemExit(f"preserve previous fixture: {target}")
        target.write_bytes(blob)
        rows.append({"path": target.name, "sha256": hashlib.sha256(blob).hexdigest(),
                     "bytes": len(blob), "children": count, "metadata": metadata})
    manifest = {
        "status": "authored_only_strict_import_and_execution_pending",
        "provenance": "original repository author; no reference implementation or acquired bytes",
        "base_sha256": hashlib.sha256(base.read_bytes()).hexdigest(),
        "author_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "contract": {"lifetime_seconds": 8, "initial_position": [.125, .25, -.5],
                     "velocity": [1, -.5, .25], "initial_rotation": 3,
                     "angular_rate": 2, "root": "one-shot prefix controller",
                     "capacity_units": "children plus one resident controller"},
        "fixtures": rows,
    }
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    main()
