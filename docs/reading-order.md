# Reading Order

Suggested order for understanding the codebase:

1. `src/protocol/vint.rs` — variable-length integers (building block, first used in var_header)
2. `src/protocol/string.rs` — length-prefixed strings (uses vint for length)
3. `src/protocol/collections.rs` — string lists, maps, set-maps (uses vint + string)
4. `src/protocol/status.rs` — status byte flags
5. `src/protocol/version.rs` — version ID decoding
6. `src/protocol/header.rs` — fixed 23-byte header (no vint, all fixed-size BE integers)
7. `src/protocol/var_header.rs` — variable header (first place vint appears on the wire)
8. `src/protocol/message.rs` — full message envelope
9. `src/capture/pcap.rs` — pcap file reading
10. `src/capture/reassembly.rs` — TCP stream reassembly
11. `src/main.rs` — CLI entry point
