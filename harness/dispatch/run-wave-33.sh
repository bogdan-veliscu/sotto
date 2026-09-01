#!/usr/bin/env bash
# Dispatch runner for parakeet-runtime (waves 33–34). See wave-33.md.
set -euo pipefail
cd "$(dirname "$0")/../.."
exec cat harness/dispatch/wave-33.md
