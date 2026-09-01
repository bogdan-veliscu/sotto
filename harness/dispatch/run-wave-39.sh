#!/usr/bin/env bash
# Dispatch runner for judge-reliability (waves 39–40). See wave-39.md.
set -euo pipefail
cd "$(dirname "$0")/../.."
exec cat harness/dispatch/wave-39.md
