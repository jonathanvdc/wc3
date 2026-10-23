"""Author original shared-programme completed-frame controls."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import struct
import zlib

CHILDREN = 1000
INSTANCES = [1, 32]
OPTIONS = ["--times", "0.5", "--benchmark-advance-frames", "64", "--fps", "64",
           "--size", "160x120", "--eye", "0,0,600", "--target", "0,0,0",
           "--msaa", "1", "--no-default-light", "--particle-stats"]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def sprite_png():
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xffffffff)
    return (b"\x89PNG\r\n\x1a\n"
            + chunk(b"IHDR", struct.pack(">IIBBBBB", 2, 2, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress((b"\0" + bytes([255] * 8)) * 2, 9))
            + chunk(b"IEND", b""))


def host_mdl(instances):
    header = '''Version { FormatVersion 1800, }
Model "Original shared immutable frame workload" {
    MinimumExtent { -200.0, -200.0, -200.0 },
    MaximumExtent { 200.0, 200.0, 200.0 },
    BoundsRadius 350.0,
}
Sequences 1 { Anim "Stand" { Interval { 0, 10000 }, } }
'''
    emitters = []
    for index in range(instances):
        emitters.append(f'''ParticleEmitterPopcorn "Independent original emitter {index}" {{
    ObjectId {index},
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
''')
    return (header + "".join(emitters)
            + f"PivotPoints {instances} {{\n"
            + "    { 0.0, 0.0, 0.0 },\n" * instances + "}\n").encode()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--motion", action="store_true", help="moving Position and authored axes, with an original atlas")
    parser.add_argument("--facing", type=int, choices=[0, 2, 3, 4, 5], default=0)
    args = parser.parse_args()
    if args.facing and not args.motion:
        parser.error("--facing requires --motion")
    repo, output = args.repo.resolve(), args.output.resolve()
    if output.exists():
        raise SystemExit("preserve previous controls; output must be fresh")
    source = repo / "crates/wc3/tests/fixtures/pkb/generate_immutable.py"
    spec = importlib.util.spec_from_file_location("original_age_author", source)
    variant = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(variant)
    base, author = variant.load_author(repo)
    variant.install_immutable_variant(author)
    sources = [Path(__file__), source, base]
    if args.motion:
        motion_source = source.with_name("generate_immutable_motion.py")
        motion_spec = importlib.util.spec_from_file_location("original_motion_author", motion_source)
        motion = importlib.util.module_from_spec(motion_spec)
        motion_spec.loader.exec_module(motion)
        motion.install_motion_variant(author, args.facing)
        image = motion.sprite_png()
        sources.append(motion_source)
    else:
        image = sprite_png()
    blob, info = author.build(CHILDREN)
    payloads = {"effect.pkb": blob, "sprite.png": image}
    for instances in INSTANCES:
        payloads[f"instances-{instances}.mdl"] = host_mdl(instances)
    output.mkdir(parents=True)
    files = {}
    for name, data in payloads.items():
        (output / name).write_bytes(data)
        files[name] = {"bytes": len(data), "sha256": digest(data)}
    manifest = {
        "status": "authored_not_strict_imported_or_rendered",
        "provenance": "original repository native writer, complete original MDLs and RGBA; no external bytes",
        "sources": {str(path): digest(path.read_bytes()) for path in sources},
        "files": files, "children_per_feature": CHILDREN, "features": INSTANCES,
        "options": OPTIONS, "routes": ["simd-jit", "immutable"],
        "programme": info, "motion": args.motion, "facing": args.facing,
        "declared_work": {
            "capacity": "default1024 total slots per feature, includes one controller +1000 children",
            "positions": ("coincident moving Float3 centers and authored axes; same geometry/overdraw on both routes"
                          if args.motion else "all zero; same transparent geometry/overdraw on both routes"),
            "interval": "capture0.5s then64 natural external1/64s frames to1.5s; child lifetime4s",
            "births": "one original burst; no new births/retirement in measured interval",
            "internal_steps": "capture may subdivide; never assume64*1000 invocations",
        },
    }
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    main()
