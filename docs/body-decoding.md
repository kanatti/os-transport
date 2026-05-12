# Body Decoding

## Overview

The message body is the payload after the fixed + variable headers. It contains the serialized fields of the action's Request or Response class. There are no field names on the wire — it's purely positional. We decode it by knowing the exact field order from the Java source.

## Architecture

```rust
trait BodyDecoder {
    fn decode(&self, data: &[u8], version: &Version) -> Vec<Field>;
}

struct Field {
    offset: usize,
    len: usize,
    name: &'static str,
    value: FieldValue,
}

enum FieldValue {
    VInt(u32),
    VLong(u64),
    String(String),
    Bool(bool),
    Byte(u8),
    Int(i32),
    Long(i64),
    StringList(Vec<String>),
    Raw(Vec<u8>),
}
```

A decoder is a function that reads the body bytes left-to-right, consuming fields in the exact order Java writes them, and returns labeled fields with their byte offsets.

## How to Write a Decoder

1. Find the Java class for the action (e.g. `ClusterHealthRequest.java`)
2. Trace the `writeTo(StreamOutput)` chain including all `super.writeTo()` calls
3. Translate each write call to a read in Rust, in order

### Example: ClusterHealthRequest

Java source chain:
```
ClusterHealthRequest.writeTo()
  → MasterNodeReadRequest.writeTo()
    → MasterNodeRequest.writeTo()
      → ActionRequest.writeTo()
        → TransportRequest.writeTo()
          → TransportMessage.writeTo()
```

Each level writes its fields, innermost first.

### Inheritance Chain

This is the hard part. Every request has a class hierarchy and each level may write fields:

```
TransportMessage.writeTo()        → (nothing in recent versions)
TransportRequest.writeTo()        → parentTaskId (NodeId + long)
ActionRequest.writeTo()           → (nothing)
MasterNodeRequest.writeTo()       → masterNodeTimeout
MasterNodeReadRequest.writeTo()   → local (boolean)
ClusterHealthRequest.writeTo()    → indices, timeout, level, waitFor*, etc.
```

The decoder must read all these in order.

### Version Conditionals

Fields appear or disappear based on version:

```java
// In Java:
if (out.getVersion().onOrAfter(Version.V_2_17_0)) {
    out.writeString(newField);
}
```

Our decoder must replicate this logic:

```rust
if version.on_or_after(2, 17, 0) {
    let (s, n) = read_string(&data[offset..])?;
    fields.push(Field { offset, len: n, name: "new_field", value: String(s) });
    offset += n;
}
```

## Decoder Registry

Map action names to their decoders:

```rust
fn get_decoder(action: &str, is_response: bool) -> Option<&dyn BodyDecoder> {
    match (action, is_response) {
        ("cluster:monitor/health", false) => Some(&ClusterHealthRequestDecoder),
        ("cluster:monitor/health", true) => Some(&ClusterHealthResponseDecoder),
        _ => None,  // raw hex dump
    }
}
```

## Priority Decoders

| Action | Request Class | Response Class | Complexity |
|--------|--------------|----------------|-----------|
| `cluster:monitor/health` | ClusterHealthRequest | ClusterHealthResponse | Low |
| `internal:coordination/fault_detection/follower_check` | FollowerCheckRequest | Empty | Low |
| `internal:coordination/fault_detection/leader_check` | LeaderCheckRequest | LeaderCheckResponse | Low |
| `internal:cluster/coordination/publish_state` | PublishClusterStateAction | PublishWithJoinResponse | Medium |
| `indices:admin/create` | CreateIndexRequest | CreateIndexResponse | Medium |
| `indices:data/read/search` | SearchRequest | SearchResponse | High |

## Fallback: Raw Hex Dump

When no decoder exists, show body as plain hex:

```
  Body (164B):
    0030: 01 0A 74 65 73 74 2D 69   6E 64 65 78 00 00 75 30
    0040: 00 0A 01 00 00 00 00 00   00 00 00 00 00 01 00 00
```

## Finding the Java Source

Action name → Java class mapping:

1. Search for the action string in the OpenSearch source:
   ```bash
   grep -r "cluster:monitor/health" ~/Code/os/OpenSearch/server/src/
   ```

2. This leads to the action registration (e.g. `ActionModule.java` or the action class itself)

3. The action class references the Request and Response types

4. Look at `writeTo()` and the `StreamInput` constructor in those types

Key source locations:
```
server/src/main/java/org/opensearch/action/
├── admin/cluster/health/
│   ├── ClusterHealthRequest.java
│   └── ClusterHealthResponse.java
├── search/
│   ├── SearchRequest.java
│   └── SearchResponse.java
└── ...

server/src/main/java/org/opensearch/cluster/coordination/
├── FollowersChecker.java      (follower_check request/response)
├── LeaderChecker.java         (leader_check request/response)
└── PublicationTransportHandler.java
```
