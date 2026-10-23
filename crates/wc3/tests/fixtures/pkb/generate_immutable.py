"""Author original immutable-age PKBs using the shared particle-runtime author.

The default output is this fixture directory. Pass --output to write elsewhere;
--check compares deterministic bytes and provenance without modifying files.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path


ORIGINAL_GENERATOR_SHA256 = "afa7d6417887c479b15f9e0bcd94681e59b9604358d23544e73aba1c270d918a"
COUNTS = [1, 8, 128, 1024]
CHANGES = [
    "child init inverseLife=.25",
    "replace child scene.time load by canonical self.lifeRatio",
    "sample coordinate direct Life rather than Life*coefficient",
    "identical selected child stages3/4/5",
]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def load_author(repo):
    source = repo / "crates/wc3/benches/fixtures/particle_runtime/generate.py"
    spec = importlib.util.spec_from_file_location("original_particle_author", source)
    author = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(author)
    return source, author


def install_immutable_variant(author):
    original_script = author.Document.script
    original_put = author.Document.put

    def script(self, stage, code, constants, externals, functions, slots, names):
        if stage == 0 and names and names[0] == "store self.lifeRatio":
            constants = [list(words) for words in constants]
            constants[1] = [author.F(0.25)]
        if stage == 3 and names and names[0] == "load scene.time":
            load = author.load(31, 0, 0)
            assert code.startswith(load)
            # Existing external1 names the canonical lifecycle Life field.
            code = author.load(31, 0, 1) + code[len(load):]
            old = author.call(
                len(externals) - 1, 0, author.reg(34, 15), [(26, author.reg(31, 7))]
            )
            new = author.call(
                len(externals) - 1, 0, author.reg(34, 15), [(26, author.reg(31, 0))]
            )
            assert code.count(old) == 1
            code = code.replace(old, new)
            names = list(names)
            names[0] = "load self.lifeRatio"
            names[15] = "sample curve(life)"
            links = [
                original_script(
                    self, selected, code, constants, externals, functions, slots, names
                )
                for selected in [3, 4, 5]
            ]
            self.age_stage_links = links
            return links[0]
        return original_script(
            self, stage, code, constants, externals, functions, slots, names
        )

    def put(self, link, fields):
        object_type = self.types[self.objects[link - 1][0]]
        if object_type == "CLayerCompileCache" and hasattr(self, "age_stage_links"):
            replacement = []
            for field, value in fields:
                if (
                    field == 10
                    and value[:4] == author.W(2)
                    and value[-4:] == author.W(self.age_stage_links[0])
                ):
                    initializer = int.from_bytes(value[4:8], "little")
                    value = author.A([initializer] + self.age_stage_links)
                replacement.append((field, value))
            fields = replacement
        return original_put(self, link, fields)

    author.Document.script = script
    author.Document.put = put


def fixtures(repo):
    source, author = load_author(repo)
    install_immutable_variant(author)
    manifest = {
        "provenance": "original direct-Life variant of repository author; encoding is not execution validation",
        "base": str(source.relative_to(repo)),
        "base_sha256": digest(source.read_bytes()),
        "generator_sha256": digest(Path(__file__).read_bytes()),
        "previous_generator_sha256": ORIGINAL_GENERATOR_SHA256,
        "changes": CHANGES,
        "fixtures": [],
    }
    results = []
    for count in COUNTS:
        blob, _ = author.build(count)
        name = f"immutable-{count}.pkb"
        results.append((name, blob))
        manifest["fixtures"].append(
            {"path": name, "bytes": len(blob), "sha256": digest(blob), "count": count}
        )
    results.append(
        ("immutable-manifest.json", (json.dumps(manifest, indent=2) + "\n").encode())
    )
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[5])
    parser.add_argument("--output", type=Path, default=Path(__file__).resolve().parent)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    output = args.output.resolve()
    results = fixtures(args.repo.resolve())
    if args.check:
        for name, data in results:
            target = output / name
            if not target.exists() or target.read_bytes() != data:
                raise SystemExit(f"original immutable fixture differs: {target}")
        return
    output.mkdir(parents=True, exist_ok=True)
    for name, data in results:
        (output / name).write_bytes(data)


if __name__ == "__main__":
    main()
