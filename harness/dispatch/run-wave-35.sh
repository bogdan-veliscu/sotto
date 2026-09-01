#!/usr/bin/env bash
# Dispatch runner for stt-worker (waves 35–36). See wave-35.md.
set -euo pipefail
cd "$(dirname "$0")/../.."
exec cat harness/dispatch/wave-35.md
