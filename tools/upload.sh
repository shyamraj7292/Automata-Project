#!/usr/bin/env bash
# Quick helper to push the latest build to the remote.
set -euo pipefail
git add -A
git commit -m "chore: update build"
git push