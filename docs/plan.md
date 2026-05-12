# os-transport: Plan

## Vision

A Rust CLI tool for inspecting OpenSearch's internal binary transport protocol (port 9300). Think `tcpdump` meets Wireshark, but purpose-built for OpenSearch. Decode the wire format into human-readable output, match request/response pairs, show latencies, and help debug cluster communication issues.

## Use Cases

1. **Debugging cluster issues** — see what actions are flying between nodes, spot errors, find slow responses
2. **Performance analysis** — measure serialization sizes, compression ratios, request latencies per action
3. **Learning/exploration** — understand what happens on the wire when you hit the REST API
4. **Testing** — verify custom plugin transport actions serialize correctly

## Features (Prioritized)

### Phase 1: Core Protocol Parser (library)

The foundation. Parse raw bytes into structured messages.

- [ ] VInt/VLong/ZLong decoders
- [ ] Fixed header parsing (23 bytes)
- [ ] Status byte flag decoding
- [ ] Version ID decoding (OS vs ES detection, semver extraction)
- [ ] Variable header parsing (thread context headers, action name)
- [ ] String decoding (VInt length prefix + UTF-8)
- [ ] Unit tests with hand-crafted byte sequences

### Phase 2: Decode CLI (offline parsing)

Parse hex input or binary files.

- [ ] `os-transport decode --hex "4553..."` — parse a single message from hex
- [ ] `os-transport decode --file message.bin` — parse from binary file
- [ ] Pretty-printed output (colored, structured)
- [ ] Show: magic, request ID, status flags, version, action name, headers, body size

### Phase 3: Pcap Reader

Parse captured traffic offline.

- [ ] `os-transport read capture.pcap` — parse all messages from a pcap file
- [ ] TCP stream reassembly (messages span packets, multiple messages per packet)
- [ ] Connection tracking (identify node pairs)
- [ ] Request/response matching by request ID
- [ ] Latency calculation (time between request and response)

### Phase 4: Live Capture

Sniff traffic in real-time.

- [ ] `os-transport capture --port 9300` — live packet capture
- [ ] Real-time output as messages arrive
- [ ] Filter by action: `--action "indices:data/read/search*"`
- [ ] Filter by node/IP: `--node 10.0.1.5`
- [ ] Summary on exit (message counts, avg latencies)

### Phase 5: Body Decompression

Handle compressed messages.

- [ ] Detect compression flag in status byte
- [ ] Deflate decompression of message body
- [ ] Show decompressed size vs wire size

### Phase 6: Stats Mode

Aggregate analysis.

- [ ] `os-transport stats capture.pcap` — summary statistics
- [ ] Messages per action (count, avg size, p50/p99 latency)
- [ ] Compression ratio per action
- [ ] Busiest node pairs
- [ ] Error rate per action
- [ ] Timeline view (messages per second)

### Phase 7: Body Decoders (stretch)

Decode well-known action bodies into readable form.

- [ ] `cluster:monitor/health` — ClusterHealthRequest/Response
- [ ] `indices:data/read/search` — SearchRequest (partial, complex)
- [ ] `internal:cluster/state` — ClusterStateRequest
- [ ] Pluggable decoder trait for adding more

## Architecture

```
src/
├── main.rs                 — CLI entry point (clap)
├── lib.rs                  — Library root
├── protocol/
│   ├── mod.rs
│   ├── vint.rs             — VInt/VLong/ZLong encoding
│   ├── header.rs           — Fixed + variable header
│   ├── status.rs           — Status byte flags
│   ├── version.rs          — Version ID ↔ semver
│   ├── message.rs          — Full message struct
│   └── string.rs           — String/collection decoding
├── capture/
│   ├── mod.rs
│   ├── pcap.rs             — Pcap file reader
│   ├── live.rs             — Live capture (libpcap)
│   └── reassembly.rs       — TCP stream reassembly
├── decode/
│   ├── mod.rs
│   ├── thread_context.rs   — Variable header (thread context)
│   └── bodies/             — Per-action body decoders
│       ├── mod.rs
│       └── cluster_health.rs
├── output/
│   ├── mod.rs
│   ├── pretty.rs           — Colored terminal output
│   ├── json.rs             — JSON output
│   └── stats.rs            — Aggregated stats
└── cli/
    ├── mod.rs
    ├── decode.rs           — decode subcommand
    ├── read.rs             — read subcommand
    ├── capture.rs          — capture subcommand
    └── stats.rs            — stats subcommand
```

## Crates

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `flate2` | Deflate decompression |
| `pcap` | Pcap file reading + live capture |
| `etherparse` | TCP/IP packet parsing |
| `colored` | Terminal colors |
| `serde` + `serde_json` | JSON output |
| `anyhow` | Error handling |

## Build Order

Start bottom-up. Each phase builds on the previous.

1. **`protocol/vint.rs`** — pure functions, easy to test, no dependencies
2. **`protocol/status.rs`** — trivial bit manipulation
3. **`protocol/version.rs`** — version ID math
4. **`protocol/string.rs`** — uses vint
5. **`protocol/header.rs`** — combines all of the above
6. **`decode/thread_context.rs`** — parses variable header using string/vint
7. **`protocol/message.rs`** — full message struct tying it together
8. **CLI `decode`** — first usable command
9. **TCP reassembly** — the hard part
10. **Pcap reader** — uses reassembly
11. **Live capture** — same logic, different input source

## Verification Strategy

- Hand-craft test bytes based on the protocol spec in `docs/protocol.md`
- Capture real traffic from a local 3-node cluster (`scripts/experimental/run.sh`)
- Compare parser output against OpenSearch TRACE logs (which show request IDs, actions, sizes)
- Cross-reference with Wireshark raw view

## Non-Goals (for now)

- Writing/sending messages (this is read-only, a parser not a client)
- Full body decoding for all actions (too many, evolve over time)
- TLS decryption (would need key material, complex)
- Windows support (Linux/macOS only)
