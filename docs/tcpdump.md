# tcpdump and pcap

## tcpdump

CLI packet capture tool. Captures raw network packets and writes them to a file or prints to stdout.

```bash
# Basic usage
tcpdump -i lo -s 65535 -w output.pcap port 9300

# Flags:
#   -i <interface>   Which network interface (lo = loopback, eth0, any)
#   -s <snaplen>     Max bytes to capture per packet (65535 = full packet)
#   -w <file>        Write to file (pcap format)
#   <filter>         BPF filter expression
```

Requires root (raw socket access).

## BPF Filters

Berkeley Packet Filter — a mini filter language compiled into kernel bytecode. Only matching packets get copied to userspace, so it's efficient.

```bash
port 9300                          # any traffic to/from port 9300
port 9300 or port 9301             # multiple ports
host 10.0.1.5 and port 9300       # specific node
src host 10.0.1.5                  # only from this IP
tcp                                # only TCP (not UDP)
```

For a local 3-node OpenSearch cluster (transport on 9300, 9301, 9302):

```bash
tcpdump -i lo -s 65535 -w capture.pcap port 9300 or port 9301 or port 9302
```

This captures all inter-node transport traffic and nothing else.

## pcap file format

**P**acket **Cap**ture. Plain binary, no compression, no indexing.

```
+---------------------------+
| Global Header (24 bytes)  |
+---------------------------+
| Packet Header (16 bytes)  |  ← packet 1
| Packet Data   (N bytes)   |
+---------------------------+
| Packet Header (16 bytes)  |  ← packet 2
| Packet Data   (N bytes)   |
+---------------------------+
| ...                       |
+---------------------------+
```

### Global Header (24 bytes)

```
Bytes 0-3:   Magic number (0xA1B2C3D4 = big-endian, 0xD4C3B2A1 = little-endian)
Bytes 4-5:   Major version (usually 2)
Bytes 6-7:   Minor version (usually 4)
Bytes 8-11:  Timezone offset (usually 0)
Bytes 12-15: Timestamp accuracy (usually 0)
Bytes 16-19: Snapshot length (max bytes per packet, e.g. 65535)
Bytes 20-23: Link-layer type (1 = Ethernet, 0 = Loopback/Null)
```

### Packet Header (16 bytes)

```
Bytes 0-3:   Timestamp seconds (Unix epoch)
Bytes 4-7:   Timestamp microseconds
Bytes 8-11:  Captured length (how many bytes actually saved)
Bytes 12-15: Original length (how many bytes were on the wire)
```

### Packet Data

Raw bytes as seen on the wire. Structure depends on link-layer type:

For Ethernet (type 1):
```
[Ethernet Header 14B] [IP Header 20B+] [TCP Header 20B+] [Payload...]
```

For Loopback (type 0, what we capture with `-i lo`):
```
[Loopback Header 4B] [IP Header 20B+] [TCP Header 20B+] [Payload...]
```

The payload is where our OpenSearch `ES` messages live.

## pcapng

"pcap next generation" — newer format used by Wireshark by default. Supports multiple interfaces, comments, name resolution blocks, etc. More complex to parse. The `pcap` Rust crate handles both.
