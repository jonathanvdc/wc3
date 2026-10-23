# Native programme eligibility coverage

The public-API example and standard-library driver retain eligibility and population observations across the native corpus. The client uses all 58 native PKBs in stable path order, each effect root, Physics/Fixed/Varying, seeds 0/42, capacity 128 and explicit ImmutableAge + Graphics. Missing payload uses explicit Zero, counter RNG and LastOccurrence stream duration. It reuses the existing corpus lifecycle: start, zero observation, 120 frames at 60 Hz, zero, stop, zero/stopped advancing/zero, restart, zero, another 120 frames, zero. Failed calls remain visible and later observations are retained.

Build `particle_specialization_audit` with Cargo under the execution features being audited. Both feature and portable builds can run it; fallback is recorded, never assumed. Pass the repository root to the example.

    python3 crates/wc3/benches/support/specialization_corpus/run.py /path/to/client /tmp/fresh-coverage-output --repo /Users/jonathanvdc/Code/wc3-mdx
    python3 crates/wc3/benches/support/specialization_corpus/run.py /path/to/client /tmp/fresh-coverage-output --repo /Users/jonathanvdc/Code/wc3-mdx --reference-manifest /path/to/previous/corpus/manifest.json --execute

Without --execute, the driver prints a pinned plan and creates nothing. Execution requires a fresh output directory. Binary, client, driver, README, Cargo inputs, the recursive Popcorn/PKB source inventory and all native hashes are guarded before/after; optional previous manifest assets must agree exactly. Raw JSONL and stderr survive failures. Raw JSONL retains exact compile/runtime errors. The summary retains diagnostic counts, typed specialization rejections (reason, original offset and field), prepared stage routes/fallbacks and observed cases. No known-admission floor silently excludes failures; ending/completion/partition checks fail incomplete output.

Prepared layer coverage is counted once per root/evolution, independent of seeds. Runtime weights are sums of observed per-layer live and authorized immutable-sample populations on advancing observations. They are exposures, NOT invoked-lane counts or elapsed work. Initial-only newborns, CPU fallback and absent immutable samples are not conflated: samples are counted only when the public immutable_sample view exists, with current authorized sample count separately recorded. Each observation preserves the actual omissions and errors; stopped advancing observations remain explicitly named.

The public API does not expose actual native-entry counts by layer/stage. `stage` rows therefore say `prepared_route`, never actual route. Actual instance-wide scalar JIT/Age/Rust packet/SIMD counters are preserved on every observation; summary adds exact cumulative deltas across start/restart, without distributing those counters over layers. Actual immutable samples are observable by layer but are not claimed to equal per-frame invocations. There is no GPU/shader eligibility or rendering proof in this CPU audit.

These source/binary hashes establish stable observed inputs, not proof that the executable was built from those sources; retain the build receipt.

No timing/field-value hashing is included: this tool answers coverage and observed populations. Existing 58-asset differential ledger remains the trajectory correctness oracle.
