# Benchmark fixture design

The fixtures provide repeatable workloads for codec and runtime changes without
requiring game files during a run. Generators under `support/` create model,
texture, and archive inputs; existing synthetic test fixtures supply focused
effect and archive compatibility cases. This guide is for contributors changing
those inputs. For commands and measurement boundaries, see the
[benchmark guide](README.md).

## Model profiles

The profiles cover small classic units, detailed units, and large buildings.
Their geometry and animation values are procedural. The profiles retain connected rigs, pivots, valid resource references,
multiple sequences, uneven track sizes, and irregular key times. These relationships
matter because uniformly repeated records can exercise different paths from
structured model data.

| Profile | Vertices | Geosets | Bones | Sequences | Rig depth | Rotation |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| `classic` | 576 | 3 | 24 | 13 | 8 | Hermite |
| `unit` | 7,488 | 8 | 114 | 27 | 17 | Linear |
| `building` | 63,990 | 27 | 35 | 6 | 4 | Linear |

The building profile is enabled by `WC3_BENCH_LARGE`. Quick mode selects the unit
profile. Each profile has an 800 layout and a modern 1800 variant; the modern
variant adds authored LODs, distinct UV sets, and four nonuniform skin influences.
The unit and building variants also exercise HD texture roles. Track lengths vary
by joint activity, and material/geoset properties animate alongside the rig.

The geometry consists of simple nondegenerate triangles. It supports codec,
preparation, and rig scaling checks, but its screen coverage and artistic appearance
do not represent a game model. Exporting it for a GPU capture helps inspect the
workload; it does not establish Warcraft III rendering fidelity.

## Texture, archive, and effect inputs

Generated textures combine flat regions, sharp boundaries, fine detail, alpha
holes, and gradients. These patterns exercise palette quantization, block
compression, and mip filtering. Supported BLP encodings are tested against the
same source pixels so their workloads remain comparable.

Archive payloads combine a deterministic structured byte pattern with seeded
noise to vary compressibility. They bracket compression behavior without
attempting to reproduce a game's distribution of file sizes. Fixed synthetic
single-unit streams exercise legacy and chained codecs. The existing StormLib
fixtures independently cover extended indexes and raw chunk digests.

PREM, PRE2, and ribbon workloads reuse the checked-in capture fixtures. These
isolate supported effects with consistent nodes, animation, resources, and
lifetimes. PREM includes its child model and texture in the repository, so its
asset loading is self-contained.

## Revise a workload

Inspect source metadata and relationships with `model_info` and the local asset
tools, then translate the relevant characteristics into synthetic parameters or
generation rules. Downloaded originals and collector tooling stay under ignored
`data/`. Original geometry, animation values, pixels, and dependency paths do not
belong in the generated fixtures, and game files must not become benchmark inputs.

Keep each profile's meaning stable. Rename a workload when changing it enough to
invalidate historical comparisons, and keep the quick runner's filters aligned
with its benchmark IDs. Run the affected suite with `-- --test` to check generation
and output validation; model generation also checks hierarchy and geometry
invariants. Inspect companion captures when changing a GPU workload.
