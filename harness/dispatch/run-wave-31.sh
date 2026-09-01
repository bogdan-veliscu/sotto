#!/usr/bin/env bash
# Dispatch runner for mixed-capture (waves 31–32). See wave-31.md.
set -euo pipefail
cd "$(dirname "$0")/../.."
exec cat harness/dispatch/wave-31.md
