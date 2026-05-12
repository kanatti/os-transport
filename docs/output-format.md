# Output Format

## Compact Mode (default)

One line per message. For quick overview and filtering.

```
[00.000] node-1 → node-2  REQ #433   243B  follower_check
[00.003] node-1 ← node-2  RSP #433    19B  (3ms)
[00.150] node-0 → node-1  REQ #180   100B  cluster:monitor/health
[00.155] node-0 ← node-1  RSP #180    63B  (5ms)
```

Format:
```
[timestamp] src → dst  REQ/RSP #id  size  action_or_latency
```

- Timestamp: relative to first packet in capture (seconds.milliseconds)
- Nodes: labeled by transport port (node-0=9300, node-1=9301, node-2=9302)
- Arrow direction: `→` for outgoing, `←` for reply on same line as originator
- Size: message length in bytes
- Action: shown for requests; latency shown for responses

## Verbose Mode (-v)

Annotated hex dump. Every byte shown with interpretation on the right.

```
[00.150] node-0 → node-1  REQ #180  cluster:monitor/health  100B

  Fixed Header:
    0000: 45 53                             magic: "ES"
    0002: 00 00 00 64                       length: 100
    0006: 00 00 00 00 00 00 00 B4           request_id: 180
    000E: 00                                status: REQ
    000F: 08 2E D8 93                       version: OS 3.7.0
    0013: 00 00 00 38                       var_header_size: 56

  Variable Header:
    0017: 00                                request_headers: 0
    0018: 00                                response_headers: 0
    0019: 00                                features: 0
    001A: 16 63 6C 75 73 74 65 72           action: "cluster:monitor/health"
          3A 6D 6F 6E 69 74 6F 72
          2F 68 65 61 6C 74 68

  Body:
    0030: 01                                index_count: 1 (vint)
    0031: 0A 74 65 73 74 2D 69 6E           indices[0]: "test-index"
          64 65 78
    003B: 00 00 75 30                       timeout_ms: 30000
    003F: 00                                level: CLUSTER (byte)
    0040: ...
```

## Layout Rules

### Hex Column

- Offset: 4 hex digits, colon, space
- Bytes: groups of up to 8 bytes per line, space-separated
- Continuation lines: indent to align under first byte (no offset)

```
    0006: 00 00 00 00 00 00 00 B4           request_id: 180
    001A: 16 63 6C 75 73 74 65 72           action: "cluster:monitor/health"
          3A 6D 6F 6E 69 74 6F 72
          2F 68 65 61 6C 74 68
```

### Annotation Column

- Starts at fixed column (after hex, padded to align)
- Format: `field_name: value`
- Only on the first hex line of a field; continuation lines are blank on the right

### Alignment

```
    OOOO: HH HH HH HH HH HH HH HH        annotation
    ^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^
    |     |                                 |
    |     hex bytes (up to 8 per line)      field: value
    |     padded to 32 chars wide
    offset (4 digits)
```

Fixed widths:
- Offset: 4 chars + ": " = 6 chars
- Hex area: 8 bytes × 3 chars = 24 chars, padded to 32
- Gap: 8 spaces
- Annotation: free-form

Total indent: 4 spaces (from section header)

### Sections

Sections are labeled and separated by blank lines:

```
  Fixed Header:
    ...

  Variable Header:
    ...

  Body:
    ...
```

Two-space indent for section headers, four-space indent for content.

## Body Display

### With decoder (known action)

Fields are labeled with names from the source code:

```
  Body:
    0030: 01                                index_count: 1 (vint)
    0031: 0A 74 65 73 74 2D 69 6E           indices[0]: "test-index"
          64 65 78
```

### Without decoder (unknown action)

Plain hex dump in 16-byte rows:

```
  Body (164B):
    0030: 01 0A 74 65 73 74 2D 69   6E 64 65 78 00 00 75 30
    0040: 00 0A 01 00 00 00 00 00   00 00 00 00 00 01 00 00
    0050: ...
```

16 bytes per row, split into two groups of 8 with extra space in the middle (like xxd).

## Color (future)

- Offset: dim/gray
- Hex bytes: white
- Annotations: green for field names, yellow for values
- Section headers: bold
- REQ: blue, RSP: green, ERR: red
- Action names: cyan
