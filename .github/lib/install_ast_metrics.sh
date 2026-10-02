#!/usr/bin/env bash
# Install a pinned ast-metrics release binary with a checksum check (report-only CI).

set -euo pipefail

readonly ast_metrics_version=0.41.1
readonly ast_metrics_url="https://github.com/ast-metrics/ast-metrics/releases/download/v${ast_metrics_version}/ast-metrics_Linux_x86_64"
readonly ast_metrics_sha256=49b4458a33f420c72fad96cfc510e371db9b4f4712830eee6b34bc83191a1e42

curl -fsSL -o ast-metrics "${ast_metrics_url}"
echo "${ast_metrics_sha256}  ast-metrics" | sha256sum -c -
install -m 755 ast-metrics /usr/local/bin/ast-metrics
