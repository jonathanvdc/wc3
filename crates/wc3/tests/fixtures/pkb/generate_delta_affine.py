"""Author a routed Delta-affine PKB with fractional births and recycled slots.

The child accumulates Rotation from its actual Delta interval. A repeated
controller trigger and quarter-second lifetime exercise admission and reuse.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[5])
    parser.add_argument("--output", type=Path, default=Path(__file__).resolve().parent)
    args = parser.parse_args()
    base = args.repo.resolve() / "crates/wc3/benches/fixtures/particle_runtime/generate.py"
    spec = importlib.util.spec_from_file_location("original_particle_author", base)
    author = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(author)
    original_function = author.Document.function
    original_fields = author.Document.fields
    original_script = author.Document.script
    original_object = author.Document.obj

    def obj(self, kind, fields=None):
        if kind == "CLayerCompileCacheRendererParticleInput" and fields:
            values = dict(fields)
            if values.get(1) == author.W(10):
                assert values[0] == author.W(100) and values[3] == author.W(31)
                fields = [(index, author.W(6) if index == 0 else value)
                          for index, value in fields]
        return original_object(self, kind, fields)

    def function(self, name, receiver, arguments, traits=44):
        if name == "generate":
            return original_function(self, "trigger", receiver, [
                (1, "bool", 4), (31, "float", 4), (0xffffffff, "pCtxS", 4),
            ], traits)
        return original_function(self, name, receiver, arguments, traits)

    def fields(self, rows):
        if any(row[0] == "coefficient" for row in rows):
            for index, (name, ty, words) in enumerate(rows):
                if name in ["self.invLife", "Rotation", "coefficient"]:
                    value = {"self.invLife": 4., "Rotation": 3., "coefficient": 2.}[name]
                    rows[index] = (name, ty, [author.F(value)])
        return original_fields(self, rows)

    def script(self, stage, code, constants, externals, functions, slots, names):
        if stage == 3 and names and names[0] == "load remainingBurst":
            constants = [list(words) for words in constants] + [[0xffffffff], [author.F(.4)]]
            old = author.call(-1, 0, author.reg(27, 5), [
                (24, author.reg(26, 1)), (26, author.reg(31, 4)),
                (10, author.reg(31, 1, 0)), (10, author.reg(31, 0, 0)),
                (10, author.reg(31, 1, 0)), (8, author.reg(0xffffffff, 2)),
            ])
            new = author.call(-1, 0, author.reg(27, 5), [
                (26, author.reg(1, 3, 0)), (10, author.reg(31, 4, 0)),
                (8, author.reg(0xffffffff, 2)),
            ])
            assert code.count(old) == 1
            code = code.replace(old, new)
            names = list(names)
            names[5] = "trigger true at fraction f32(.4) each actual root invocation"
        elif stage == 3 and names and names[0] == "load scene.time":
            externals = list(externals) + [self.external("scene.dt", 31, 1, 2)]
            delta = len(externals) - 1
            code = b"".join([
                author.load(31, 0, 11),  # canonical Rotation field10
                author.load(31, 1, 8),   # canonical coefficient field7
                author.load(31, 2, delta),
                author.binary(2, 31, 1, author.reg(31, 1), author.reg(31, 2)),
                author.binary(0, 31, 0, author.reg(31, 0), author.reg(31, 1)),
                author.store(author.reg(31, 0), 11),
            ])
            constants = []
            functions = []
            slots = 3
            names = ["load Rotation", "load coefficient", "load scene.dt",
                     "coefficient*Delta", "Rotation+increment", "store Rotation"]
        return original_script(self, stage, code, constants, externals, functions, slots, names)

    author.Document.function = function
    author.Document.fields = fields
    author.Document.script = script
    author.Document.obj = obj
    blob, info = author.build(1)
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    target = output / "delta-affine.pkb"
    if target.exists():
        raise SystemExit("preserve old fixture; use a fresh output directory")
    target.write_bytes(blob)
    manifest = {
        "status": "authored_not_strict_imported_or_executed",
        "provenance": "original repository native author, no acquired or reference implementation bytes",
        "base_sha256": hashlib.sha256(base.read_bytes()).hexdigest(),
        "generator_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "file": {"path": target.name, "sha256": hashlib.sha256(blob).hexdigest(), "bytes": len(blob)},
        "controls": {"root_layer": 0, "child_layer": 1, "child_rotation_field": 10,
                     "root_trigger_fraction": "f32(.4)", "root_repeats": True,
                     "child_initial_rotation": 3, "rate": 2, "inverse_life": 4,
                     "expected_first_birth": ".25 * f64(f32(.4))",
                     "expected_first_integrated_delta": "f64(f32(.25-first_birth))"},
        "native_author_metadata": info,
    }
    (output / "delta-affine-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    main()
