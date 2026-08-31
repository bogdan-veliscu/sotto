#!/usr/bin/env bash
set -euo pipefail
cd /Users/bogdan/kiro/sotto
PROMPT=$(cat /Users/bogdan/kiro/sotto/harness/dispatch/wave-13.md)
exec kiro-cli --v3 chat --trust-all-tools --agent scribe "$PROMPT"
