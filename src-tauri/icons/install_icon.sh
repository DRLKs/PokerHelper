#!/usr/bin/env bash
# Decode icon.b64 into icon.png (overwrites if exists). Works on systems where `base64 -d` is available.
set -euo pipefail
cd "$(dirname "$0")"
if base64 -d icon.b64 > icon.png 2>/dev/null; then
	:
elif base64 --decode icon.b64 > icon.png 2>/dev/null; then
	:
else
	echo "No usable base64 decode found." >&2
	exit 1
fi
chmod 0644 icon.png
echo "Wrote $(pwd)/icon.png"
