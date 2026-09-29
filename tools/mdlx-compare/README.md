# MDLX comparison

This optional workflow runs wc3 and a pinned WhiteoutLib revision independently
against original MDX files. It retains both MDL outputs, parse diagnostics,
canonicalized outputs, and diffs. It does not modify the input corpus or add an
oracle dependency to the ordinary Rust tests.

## Setup

Requires Python 3.9+, Git, CMake 3.20+, Ninja, a C++20 compiler, and Rust. Fetch
WhiteoutLib into an ignored directory and check out the revision in
[whiteout.lock.json](whiteout.lock.json):

```sh
git clone https://github.com/FernandoS27/WhiteoutLib.git target/mdlx-compare/whiteout
git -C target/mdlx-compare/whiteout checkout d27bb3b7a50eeea22d311df18af90acd5896247f
python3 tools/mdlx-compare/compare.py build --source target/mdlx-compare/whiteout
```

The build requires a clean checkout at exactly the pinned commit. It writes
`adapter.json` containing the source revision and executable/adapter hashes.
The runner verifies this metadata before using the adapter. Rebuild after
changing the adapter, CMake file, or pin. CMake builds the upstream library;
the first build takes longer than subsequent comparisons.

For a custom compiler or SDK, pass repeatable CMake arguments:

```sh
python3 tools/mdlx-compare/compare.py build --source target/mdlx-compare/whiteout \
  --cmake-arg=-DCMAKE_CXX_COMPILER=/path/to/clang++ \
  --cmake-arg=-DCMAKE_OSX_SYSROOT=/path/to/MacOSX.sdk
```

If Rust selects an incompatible macOS SDK too, set `SDKROOT` to the matching
SDK path for the `run` command. This changes only that process's environment.

Whiteout uses its normal automatic upgrade mode. At the pinned revision this
upgrades versions 900/1000 to 1200, leaving version 800 and versions 1100+
unchanged. wc3 applies strict conversion to the same target before MDL output;
equivalent camera variants are normalized even when the version stays unchanged.
Conversion reports are retained in each stage’s diagnostics, including when a
later MDL write fails. Unsupported upgrades are reported as failures rather than
silently discarding fields. Whiteout parser issues also require review.

## Run

From the repository root:

```sh
python3 tools/mdlx-compare/compare.py run
```

By default this scans all MDX files recursively under `data/hive-workshop-models`
and runs both engine and Hive dialects. No texture files are needed. Options:

```sh
python3 tools/mdlx-compare/compare.py run --corpus /absolute/path/to/models \
  --dialects hive --timeout 60 --results target/mdlx-compare/my-run
```

Each subprocess has its own timeout (30 seconds by default). A crash, failed
conversion, or unreadable output is recorded; the remaining stages/models still
run. Setup and internal runner errors return exit code 2. Findings are informational by default;
`--fail-on-review` returns 1 if any case needs review. Results paths must be new,
so previous evidence cannot be accidentally overwritten.

Outputs default to `target/mdlx-compare/results/<UTC timestamp>/`:

```text
run.json                  # pin, hashes, wc3 revision/dirty state, options
summary.json              # complete per-model outcomes and diagnostics
README.md                 # readable outcome table with report links
001-<input hash>/engine/
  ours.mdl
  oracle.mdl
  ours.canonical.mdl
  oracle.canonical.mdl
  ours.oracle-reparsed.mdl
  ours.cross.canonical.mdl
  oracle.reparsed.mdl
  oracle.self.canonical.mdl
  conversion.diff         # present when canonical outputs differ
  cross-read.diff          # present when cross-reading changes values
  report.json             # input path/hash/size/version, stages, comparisons
```

Outputs exist only for stages that produced them. Reports include commands,
exit codes, stdout, and diagnostics. The input path in each model report is
relative to the corpus root recorded in `run.json`.

## What the comparison proves

Both converters receive the original binary input. wc3 reads each output and
writes it in the same dialect, normalizing whitespace, property spelling,
property order, number formatting, and omitted defaults through its actual
codecs. The canonical outputs are compared without float tolerances or blanket
removal of fields. IDs, references, flags, values, and animation keys remain
observable. A unified diff localizes discrepancies.

The adapter also parses our MDL with Whiteout and writes it back to MDL. wc3
canonicalizes that output for comparison with ours. A separate Whiteout
self-round-trip check helps identify discrepancies in the oracle itself.
Our own MDL output must be stable when read and rewritten.

Canonical equality is a conservative comparison, not a general semantic
isomorphism checker. Record order remains significant; animation channels
use the codec’s canonical property order. Alternate but equivalent record
organization can produce a difference. Text conversion can
normalize NaN payloads and cannot represent all original binary data. A
conversion failure is recorded, not presumed to be a bug or an intentional
restriction. Agreement does not prove both converters retained every binary
value: retain the independent byte-preservation corpus checks too.

Review failures as conversion refusals, parse failures, canonical differences,
or oracle parser issues. Neither implementation automatically wins. Record
confirmed expectations in ordinary tests rather than accepting all current
oracle output as a golden baseline.

## Turn a finding into a fixture

1. Identify the exact field/chunk and assertion from the report and diff.
2. Extract original bytes for that record, retaining the version context. For
   decoding failures, reduce raw bytes rather than decoding/re-encoding first.
3. Remove unrelated chunks, records, mesh data, and keyframes while checking
   that the same specific discrepancy still occurs. Repair references if the
   test depends on a coherent whole model.
4. Verify the expected behavior against the original bytes and format evidence.
5. Commit the small input and reviewed expected MDL/field assertions under
   `crates/wc3/tests/fixtures`, with a default-suite regression test.
6. Record the source URL, author, original SHA-256, oracle revision, reduction
   steps, behavior protected, and redistribution terms in fixture notes.

No automatic reducer or fixture promotion is included: first use the reports
to choose a concrete disagreement and its reduction predicate. Full models,
third-party source, build artifacts, and compatibility reports stay local.

Runner checks (no oracle installation required):

```sh
python3 -m unittest discover -s tools/mdlx-compare -p 'test_*.py' -v
```
