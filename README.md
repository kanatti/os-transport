# os-transport

A Rust CLI tool for parsing OpenSearch's internal binary transport protocol (port 9300).

Captures and decodes node-to-node communication — action names, request/response pairs, message sizes, thread context headers — directly from pcap files or raw bytes.

## Usage

```bash
# Parse a pcap capture
os-transport read capture.pcap

# Output:
:51874 → :9302  #433    REQ   243B  OS 3.7.0  internal:coordination/fault_detection/follower_check
:9302 → :51874  #433    RSP    19B  OS 3.7.0
:33322 → :9301  #180    REQ   100B  OS 3.7.0  cluster:monitor/health
:9301 → :33322  #180    RSP    63B  OS 3.7.0
:33352 → :9301  #187    REQ   133B  OS 3.7.0  indices:admin/create
    _system_index_access_allowed: false
```

```bash
# Decode a single message from hex
os-transport decode "4553 000000F3 00000000000001B1 00 082ED893 00000038 ..."
```

## Capturing Traffic

```bash
# Capture transport traffic from a local 3-node cluster
sudo tcpdump -i lo -s 65535 -w capture.pcap port 9300 or port 9301 or port 9302
```

A helper script is included:

```bash
./scripts/capture.sh                        # capture until Ctrl+C
./scripts/capture.sh output.pcap 30         # capture for 30 seconds
```

## What It Parses

Each message on the wire has this structure:

```
Fixed Header (23B)     → magic "ES", length, request ID, status flags, version
Variable Header        → thread context headers, action name
Body                   → action-specific payload (serialized via StreamOutput)
```

The tool fully decodes the envelope (headers + action name) and shows raw bytes for the body. Body decoders for specific actions are planned.

## Building

```bash
cargo build --release
```

## Protocol Details

See `docs/` for full protocol documentation:

- `docs/protocol.md` — wire format, encoding rules, byte-level examples
- `docs/actions.md` — complete catalog of 200+ transport actions by category
- `docs/body-decoding.md` — approach for decoding action-specific payloads
- `docs/output-format.md` — output format specification
- `docs/reading-order.md` — suggested order for reading the source
- `docs/tcpdump.md` — capturing traffic with tcpdump

## Understanding OpenSearch Through Actions

Every REST API call, cluster event, or internal coordination step becomes one or more transport actions on the wire. Each action carries a serialized payload that reveals how the subsystem works internally.

By decoding these payloads action-by-action, you learn the system from the inside out:
- **Search:** how queries scatter to shards, what the query/fetch phases carry
- **Indexing:** how bulk requests fan out, how replicas get written
- **Coordination:** how leader election, state publishing, and fault detection work
- **Recovery:** how shards rebuild — file lists, chunks, translog replay

The approach: trigger an action (REST call or cluster event), capture the traffic, trace the Java serialization code, implement a decoder, verify against real bytes. Each decoded action is a new piece of the architecture understood.
