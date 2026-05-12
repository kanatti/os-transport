#!/bin/bash
# Capture OpenSearch transport traffic from a local cluster.
# Usage: ./capture.sh [output_file] [duration_seconds]
#
# Requires: sudo (for raw packet capture), tcpdump
# Assumes: local cluster on ports 9300, 9301, 9302

OUTPUT="${1:-testdata/captures/basic.pcap}"
DURATION="${2:-}"

mkdir -p "$(dirname "$OUTPUT")"

echo "Capturing OpenSearch transport traffic on ports 9300-9302..."
echo "Output: $OUTPUT"
echo "Press Ctrl+C to stop (or will stop after ${DURATION:-∞} seconds)"
echo ""

TIMEOUT_FLAG=""
if [ -n "$DURATION" ]; then
    TIMEOUT_FLAG="-G $DURATION -W 1"
fi

sudo tcpdump -i lo -s 65535 $TIMEOUT_FLAG -w "$OUTPUT" \
  port 9300 or port 9301 or port 9302

echo ""
echo "Done. Captured $(du -h "$OUTPUT" | cut -f1) of data."
echo "Quick peek: xxd $OUTPUT | head -20"
