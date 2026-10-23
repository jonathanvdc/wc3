"""Author original moving immutable PKBs and optional existing-capture assets.

Extends generate_immutable.py without changing its four fixture outputs. Default
counts are 128 and 1000, with camera-facing geometry. --facing 0,2,3,4,5 adds
independently labelled authored-axis controls. Generation is not strict import,
execution, or rendering validation; those steps belong to the central runner.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import struct
import zlib

COUNTS = [128, 1000]
COEFFICIENT = 3.5
POSITION = [-0.4, -0.15, 0.0]
VELOCITY = [0.8, 0.3, 0.2]
AXIS0 = [0.0, 2.0, 0.0]
AXIS0_CHANGE = [0.0, 0.0, 1.0]
AXIS1 = [1.0, 0.0, 0.0]
AXIS1_CHANGE = [0.0, 0.5, 0.0]
TIMES = [0.125, 0.75, 1.75, 3.25]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def module(path):
    spec = importlib.util.spec_from_file_location("original_immutable_variant", path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def install_motion_variant(author, facing):
    original_fields = author.Document.fields
    original_script = author.Document.script
    original_obj = author.Document.obj
    original_put = author.Document.put

    def fields(self, rows):
        if any(row[0] == "coefficient" for row in rows):
            assert [row[0] for row in rows][-1] == "TextureID"
            for index, (name, native_type, words) in enumerate(rows):
                if name == "coefficient":
                    rows[index] = (name, native_type, [author.F(COEFFICIENT)])
                elif name in ["basePosition", "Position"]:
                    rows[index] = (name, native_type, [author.F(v) for v in POSITION])
            # Append only: all existing canonical field/external indices remain.
            rows.extend([
                ("Axis0", 33, [author.F(v) for v in AXIS0]),
                ("Axis1", 33, [author.F(v) for v in AXIS1]),
            ])
        return original_fields(self, rows)

    def script(self, stage, code, constants, externals, functions, slots, names):
        if stage == 3 and names and names[0] == "load scene.time":
            constants = [list(words) for words in constants]
            velocity, axis0, axis0_change, axis1, axis1_change = range(len(constants), len(constants) + 5)
            constants.extend([[author.F(v) for v in vector] for vector in [
                VELOCITY, AXIS0, AXIS0_CHANGE, AXIS1, AXIS1_CHANGE,
            ]])
            old = author.store(author.reg(33, 1), 9)
            replacement = b"".join([
                author.binary(2, 33, 17, author.reg(33, velocity, 0), author.reg(31, 0)),
                author.binary(0, 33, 18, author.reg(33, 1), author.reg(33, 17)),
                author.store(author.reg(33, 18), 9),
                author.binary(2, 33, 19, author.reg(33, axis0_change, 0), author.reg(31, 0)),
                author.binary(0, 33, 19, author.reg(33, axis0, 0), author.reg(33, 19)),
                author.store(author.reg(33, 19), 15),
                author.binary(2, 33, 20, author.reg(33, axis1_change, 0), author.reg(31, 0)),
                author.binary(0, 33, 20, author.reg(33, axis1, 0), author.reg(33, 20)),
                author.store(author.reg(33, 20), 16),
            ])
            assert code.count(old) == 1
            code = code.replace(old, replacement)
            texture = author.store(author.reg(31, 4, 0), 14)
            assert code.count(texture) == 1
            code = code.replace(texture, author.store(author.reg(31, 7), 14))
            names = list(names)
            at = names.index("store Position")
            names[at:at + 1] = [
                "velocity*Life", "basePosition+velocity*Life", "store Position",
                "axis0Change*Life", "axis0Base+change", "store Axis0",
                "axis1Change*Life", "axis1Base+change", "store Axis1",
            ]
            names[-1] = "store TextureID=Life*coefficient"
            slots = 21
        # The prior immutable wrapper changes scene.time to Life and installs
        # identical selected Physics/Fixed/Varying scripts after our edits.
        return original_script(self, stage, code, constants, externals, functions, slots, names)

    def obj(self, name, fields=None):
        if name == "CLayerCompileCacheRendererParticleInput" and fields:
            values = dict(fields)
            if values.get(0) == author.W(100) and values.get(1) == author.W(10):
                assert self.strings[int.from_bytes(values[2], "little")] == b"Rotation"
                fields = [(0, author.W(6)), (1, author.W(10))]
        return original_obj(self, name, fields)

    def put(self, link, fields):
        if self.types[self.objects[link - 1][0]] == "CLayerCompileCacheRenderer":
            properties = []
            for name, numeric in [
                ("BillboardingMode", [facing, 0, 0, 0]),
                ("Atlas", [1, 0, 0, 0]),
                ("Atlas.Source", [1, 0, 0, 0]),
                ("Atlas.SubDiv", [2, 2, 0, 0]),
            ]:
                properties.append(self.obj("CLayerCompileCacheRendererProperty", [
                    (0, self.string(name)), (1, author.W(0)),
                    (2, b"".join(author.W(word) for word in numeric)),
                ]))
            axes = [self.obj("CLayerCompileCacheRendererParticleInput", [
                (0, author.W(semantic)), (1, author.W(field)),
            ]) for semantic, field in [(4, 14), (5, 15)]]
            replacement = []
            for field, value in fields:
                if field in [1, 2]:
                    count = int.from_bytes(value[:4], "little")
                    assert len(value) == (count + 1) * 4
                    links = [int.from_bytes(value[i:i + 4], "little") for i in range(4, len(value), 4)]
                    value = author.A(links + (axes if field == 1 else properties))
                replacement.append((field, value))
            fields = replacement
        return original_put(self, link, fields)

    author.Document.fields = fields
    author.Document.script = script
    author.Document.obj = obj
    author.Document.put = put


def sprite_png():
    colors = [(255, 64, 16, 255), (32, 255, 64, 192), (32, 64, 255, 128), (255, 224, 32, 64)]
    pixels = bytearray()
    for y in range(64):
        pixels.append(0)
        for x in range(64):
            r, g, b, alpha = colors[(y // 32) * 2 + x // 32]
            if x % 32 < 8 and y % 32 < 12:
                alpha = 0
            pixels.extend((r, g, b, alpha))

    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xffffffff)

    return (b"\x89PNG\r\n\x1a\n"
            + chunk(b"IHDR", struct.pack(">IIBBBBB", 64, 64, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(bytes(pixels), 9)) + chunk(b"IEND", b""))


def host_mdl(count, facing):
    return f'''Version {{ FormatVersion 1800, }}
Model "Original moving immutable {count} facing {facing}" {{
    MinimumExtent {{ -200.0, -200.0, -200.0 }},
    MaximumExtent {{ 200.0, 200.0, 200.0 }},
    BoundsRadius 350.0,
}}
Sequences 1 {{ Anim "Stand" {{ Interval {{ 0, 10000 }}, }} }}
ParticleEmitterPopcorn "Original moving dynamic axes" {{
    ObjectId 0,
    Unshaded,
    SortPrimsFarZ,
    static LifeSpan 1.0,
    static EmissionRate 1.0,
    static Speed 1.0,
    static Color {{ 1.0, 1.0, 1.0 }},
    static Alpha 1.0,
    Path "effect.pkfx",
    AnimVisibilityGuide "Always=on",
}}
PivotPoints 1 {{ {{ 0.0, 0.0, 0.0 }}, }}
'''.encode()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[5])
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--counts", default="128,1000")
    parser.add_argument("--facing", default="0")
    parser.add_argument("--capture-output", type=Path)
    args = parser.parse_args()
    counts = [int(value) for value in args.counts.split(",")]
    modes = [int(value) for value in args.facing.split(",")]
    if not counts or any(count < 1 or count > 1023 for count in counts):
        raise SystemExit("counts must fit the default 1024-slot pool including its root")
    if not modes or any(mode not in [0, 2, 3, 4, 5] for mode in modes):
        raise SystemExit("supported controlled facing modes are 0,2,3,4,5")
    repo, output = args.repo.resolve(), args.output.resolve()
    capture = args.capture_output.resolve() if args.capture_output else None
    if output.exists() or (capture is not None and capture.exists()):
        raise SystemExit("outputs must be fresh; preserve previous evidence")
    original = Path(__file__).resolve().with_name("generate_immutable.py")
    variant = module(original)
    output.mkdir(parents=True)
    if capture is not None:
        capture.mkdir(parents=True)
    manifest = {
        "status": "encoded_not_strict_imported_or_executed",
        "provenance": "original repository native author and original stdlib RGBA image; no acquired/reference bytes",
        "generator_sha256": digest(Path(__file__).read_bytes()),
        "immutable_generator_sha256": digest(original.read_bytes()),
        "fixtures": [], "capture_jobs": [],
        "native_contract": {"Position": {"base": POSITION, "velocity_per_life": VELOCITY},
                            "Axis0": {"base": AXIS0, "change_per_life": AXIS0_CHANGE},
                            "Axis1": {"base": AXIS1, "change_per_life": AXIS1_CHANGE},
                            "rotation_semantic": 6, "axis_semantics": [4, 5],
                            "lifetime_seconds": 4, "coefficient": COEFFICIENT,
                            "atlas": [2, 2], "TextureID": "Life*3.5",
                            "coordinate_note": "native Float3 values; renderer applies its ordinary model basis exactly once"},
    }
    for facing in modes:
        base, author = variant.load_author(repo)
        variant.install_immutable_variant(author)
        install_motion_variant(author, facing)
        manifest["base_author_sha256"] = digest(base.read_bytes())
        for count in counts:
            blob, info = author.build(count)
            stem = f"immutable-motion-{count}-facing{facing}"
            (output / f"{stem}.pkb").write_bytes(blob)
            entry = {"path": f"{stem}.pkb", "sha256": digest(blob), "bytes": len(blob),
                     "count": count, "facing": facing, "fields": info["child_field_order"],
                     "externals": info["child_external_order"], "scripts": info["scripts"]}
            if capture is not None:
                case = capture / stem
                case.mkdir()
                payloads = {"effect.pkb": blob, "model.mdl": host_mdl(count, facing), "sprite.png": sprite_png()}
                entry["capture_files"] = {}
                for name, data in payloads.items():
                    (case / name).write_bytes(data)
                    entry["capture_files"][name] = {"sha256": digest(data), "bytes": len(data)}
                for route in ["interpreter", "immutable"]:
                    argv = [str(case / "model.mdl"), str(capture / "captures" / f"{stem}-{route}"),
                            "--asset-root", str(case), "--times", ",".join(map(str, TIMES)),
                            "--fps", "64", "--size", "640x480", "--eye", "0,0,600",
                            "--target", "0,0,0", "--msaa", "1", "--no-default-light",
                            "--particle-stats", "--particle-execution", route]
                    manifest["capture_jobs"].append({"case": stem, "route": route, "argv": argv})
            manifest["fixtures"].append(entry)
    (output / "immutable-motion-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    if capture is not None:
        (capture / "capture-argv.json").write_text(json.dumps(manifest["capture_jobs"], indent=2) + "\n")
        (capture / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    main()
