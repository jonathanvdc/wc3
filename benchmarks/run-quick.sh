#!/usr/bin/env bash
# Representative optimized measurements for pull requests; no performance gate.
set -euo pipefail

task_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd -- "$task_root"

filter='^model/unit/(mdx/read/(800|1800)|mdx/write/1800|mdl/(read|write)/1800|geometry/access|resources|convert/800-1800)$'
filter+='|^animation/(scalar|vector|quaternion)/(step|linear|hermite|bezier)/(sequential|scattered)/64$'
filter+='|^blp/(container/read|decode/base|encode/chain)/(indexed1-a8|jpeg1|dxt5|bgra)/256$'
filter+='|^mpq/(extract/shared/(stored|zlib|bzip2)/encrypted/structured/65536|write/zlib/encrypted/structured/65536|open/shared/(0|3)|lookup/(entry-127|missing)/(0|3)|single-unit/(pkware-implode|huffman-zero|adpcm-stereo|lzma-sized)|file/warm-cache)$'
filter+='|^assets/(direct/shared|adapter/single|batch/limit-(1|4)/(1|16)|overlay/(last-hit|miss)/(1|16))$'
filter+='|^evaluation/(pose/(64|256)|node/(generic|warcraft3)|project_1024_triangles|node_lookup/(linear|prepared)/(64|256|1024))$'
filter+='|^preparation/unit/(decode/(mdx|mdl)|assets|pose/bake)$'
filter+='|^runtime/(unit/(spawn|despawn|update/frame-60hz)/32|effects/(prem|pre2|ribbon)/frame-60hz/32)$'

exec env WC3_BENCH_QUICK=1 CRITERION_HOME="${CRITERION_HOME:-$task_root/target/criterion/quick}" cargo bench --workspace --all-features --locked \
    --bench model --bench animation --bench evaluation --bench blp --bench mpq \
    --bench assets --bench preparation --bench runtime -- \
    "$filter" --sample-size 10 --warm-up-time 0.05 --measurement-time 0.1 \
    --nresamples 1000 --noplot "$@"
