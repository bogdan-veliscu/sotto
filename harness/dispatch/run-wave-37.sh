#!/usr/bin/env bash
# Dispatch runner for source-picker (waves 37–38). See wave-37.md.
set -euo pipefail
cd "$(dirname "$0")/../.."
exec cat harness/dispatch/wave-37.md
