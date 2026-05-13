# os-transport: Plan

## Vision

A Rust CLI tool for inspecting OpenSearch's internal binary transport protocol (port 9300). Think `tcpdump` meets Wireshark, but purpose-built for OpenSearch. Decode the wire format into human-readable output, match request/response pairs, show latencies, and help debug cluster communication issues.

## Status

### Done

- [x] VInt/VLong/ZLong decoders
- [x] Fixed header parsing (23 bytes)
- [x] Status byte flags (u8 wrapper with methods)
- [x] Version ID decoding (OS vs ES detection, semver extraction)
- [x] Variable header parsing (thread context headers, action name)
- [x] String decoding (VInt length prefix + UTF-8)
- [x] Collection helpers (string_list, string_map, string_set_map)
- [x] Full message envelope parser
- [x] CLI: `os-transport decode <hex>` — parse single message
- [x] CLI: `os-transport read <pcap>` — parse all messages from pcap
- [x] Pcap file reading (pcap-parser + etherparse)
- [x] TCP stream reassembly (length-based, no magic scanning)
- [x] 32 unit tests passing
- [x] Test data: basic.pcap (292 messages, 3-node OS 3.7.0 cluster)
- [x] Helper scripts (capture.sh, peek.py, actions.py)
- [x] Protocol docs (protocol.md, body-decoding.md, output-format.md)
- [x] Full action catalog (docs/actions.md)

### Next Up

- [ ] Body decoders (see Body Decoding section below)
- [ ] Verbose hex-annotated output mode (`-v`)
- [ ] Request/response matching with latency calculation
- [ ] Filtering (`--action`, `--exclude`)
- [ ] Stats mode (aggregate counts, sizes, latencies per action)
- [ ] Live capture mode
- [ ] Decompression (deflate body when compressed flag set)

## Body Decoding

### Approach: Action-Driven Exploration

Each action is a window into a subsystem. To implement a body decoder:

1. **Trigger the action** — find what REST call or cluster event produces it
2. **Capture it** — tcpdump the transport traffic
3. **Trace the Java source** — read `writeTo()`/`StreamInput` constructor
4. **Implement decoder** — translate field-by-field to Rust
5. **Verify** — decode captured bytes, check fields make sense

This is also how we learn internals — understanding the payload means understanding the subsystem.

### How to Trigger Actions

| Action | How to trigger |
|--------|---------------|
| `internal:transport/handshake` | Any new node connection (start a node) |
| `internal:coordination/fault_detection/follower_check` | Always running (leader → followers, every 1s) |
| `internal:coordination/fault_detection/leader_check` | Always running (followers → leader, every 1s) |
| `cluster:monitor/health` | `curl localhost:9200/_cluster/health` |
| `cluster:monitor/state` | `curl localhost:9200/_cluster/state` |
| `cluster:monitor/nodes/stats` | `curl localhost:9200/_nodes/stats` |
| `cluster:monitor/nodes/info` | `curl localhost:9200/_nodes` |
| `indices:admin/create` | `curl -X PUT localhost:9200/my-index` |
| `indices:admin/delete` | `curl -X DELETE localhost:9200/my-index` |
| `indices:admin/mapping/put` | `curl -X PUT localhost:9200/my-index/_mapping -d '{...}'` |
| `indices:admin/refresh` | `curl -X POST localhost:9200/my-index/_refresh` |
| `indices:data/write/index` | `curl -X POST localhost:9200/my-index/_doc -d '{...}'` |
| `indices:data/write/bulk` | `curl -X POST localhost:9200/_bulk --data-binary @bulk.json` |
| `indices:data/read/search` | `curl localhost:9200/my-index/_search` |
| `indices:data/read/get` | `curl localhost:9200/my-index/_doc/1` |
| `cluster:admin/snapshot/create` | `curl -X PUT localhost:9200/_snapshot/repo/snap1` |
| `internal:cluster/coordination/publish_state` | Any cluster state change (create index, add node) |
| `internal:index/shard/recovery/*` | Create index with replicas, or restart a node |
| `indices:admin/publishCheckpoint` | Segment replication enabled + indexing |

### Decoder Priority

**Tier 0 — Generic (all messages):**
- Ping (no body, just detect it)
- Error/exception response (recursive exception format)
- TaskId prefix (all requests start with this)

**Tier 1 — High frequency (always on wire):**
- `internal:transport/handshake` req/rsp
- `internal:coordination/fault_detection/follower_check` req/rsp
- `internal:coordination/fault_detection/leader_check` req/rsp

**Tier 2 — Cluster ops:**
- `cluster:monitor/health` req/rsp
- `cluster:monitor/state` req/rsp
- `internal:cluster/coordination/publish_state` req/rsp
- `internal:cluster/coordination/commit_state` req/rsp

**Tier 3 — Search path:**
- `indices:data/read/search` req/rsp
- `indices:data/read/search[phase/query]` req/rsp
- `indices:data/read/search[phase/fetch/id]` req/rsp
- `indices:data/read/search[can_match]` req/rsp

**Tier 4 — Write path:**
- `indices:data/write/bulk` req/rsp
- `indices:data/write/index` req/rsp
- `indices:admin/create` req/rsp

**Tier 5 — Recovery:**
- `internal:index/shard/recovery/start_recovery` req/rsp
- `internal:index/shard/recovery/file_chunk` req/rsp
- `internal:index/shard/recovery/translog_ops` req/rsp
- `internal:index/shard/recovery/finalize` req/rsp

**Tier 6 — Everything else:** raw hex fallback

### Body Format by Message Type

| Type | Condition | Body starts with |
|------|-----------|-----------------|
| Ping | msg_len field = -1 (0xFFFFFFFF) | No body at all |
| Action request | `!is_response` | TaskId (string + optional long) + action fields |
| Action response | `is_response && !is_error` | Action-specific fields (no base prefix) |
| Error response | `is_response && is_error` | Exception format (type-tagged, recursive) |

## Architecture

```
src/
├── main.rs                 — CLI entry point (clap)
├── protocol/
│   ├── mod.rs              — pub uses
│   ├── vint.rs             — VInt/VLong/ZLong encoding
│   ├── string.rs           — Length-prefixed strings
│   ├── collections.rs      — String list/map/set-map helpers
│   ├── status.rs           — Status byte flags (u8 wrapper)
│   ├── version.rs          — Version ID ↔ semver
│   ├── header.rs           — Fixed 23-byte header
│   ├── var_header.rs       — Variable header (thread context, action)
│   └── message.rs          — Full message envelope
├── capture/
│   ├── mod.rs
│   ├── pcap.rs             — Pcap file reader
│   └── reassembly.rs       — TCP stream reassembly
└── body/                   — (planned) Per-action body decoders
    ├── mod.rs              — Decoder trait, registry, Field/FieldValue types
    ├── error.rs            — Exception decoder
    ├── handshake.rs        — Transport handshake
    ├── cluster_health.rs   — cluster:monitor/health
    └── ...
```

## Non-Goals (for now)

- Writing/sending messages (read-only parser, not a client)
- Full body decoding for all 200+ actions (grow over time)
- TLS decryption (needs key material)
- Windows support (Linux/macOS only)
