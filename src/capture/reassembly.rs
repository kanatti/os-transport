use std::collections::HashMap;
use crate::protocol::header::{FIXED_HEADER_SIZE, MAGIC};
use crate::protocol::message::Message;

/// One endpoint of a TCP connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Endpoint {
    pub addr: [u8; 4],
    pub port: u16,
}

impl std::fmt::Display for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}.{}:{}", self.addr[0], self.addr[1], self.addr[2], self.addr[3], self.port)
    }
}

/// A TCP connection identified by both endpoints (ordered so src < dst for dedup).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionKey {
    pub src: Endpoint,
    pub dst: Endpoint,
}

/// A half-connection: one direction of a TCP stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct HalfKey {
    src: Endpoint,
    dst: Endpoint,
}

/// A parsed message with metadata about where it came from.
#[derive(Debug, Clone)]
pub struct ParsedMessage {
    pub message: Message,
    pub src: Endpoint,
    pub dst: Endpoint,
    pub timestamp_us: u64,
}

/// Reassembles TCP streams and extracts OS transport messages.
pub struct TcpReassembler {
    /// Buffer per half-connection (one direction).
    streams: HashMap<HalfKey, StreamBuffer>,
    /// Parsed messages in order.
    pub messages: Vec<ParsedMessage>,
}

struct StreamBuffer {
    data: Vec<u8>,
    /// Timestamp of first packet (used for messages parsed from this chunk).
    first_ts: u64,
}

impl TcpReassembler {
    pub fn new() -> Self {
        TcpReassembler {
            streams: HashMap::new(),
            messages: Vec::new(),
        }
    }

    /// Feed a TCP payload into the reassembler.
    pub fn add_payload(&mut self, src: Endpoint, dst: Endpoint, payload: &[u8], timestamp_us: u64) {
        if payload.is_empty() {
            return;
        }

        let key = HalfKey { src, dst };
        let stream = self.streams.entry(key.clone()).or_insert_with(|| StreamBuffer {
            data: Vec::new(),
            first_ts: timestamp_us,
        });

        stream.data.extend_from_slice(payload);

        // Try to parse complete messages from the buffer
        self.drain_messages(&key);
    }

    fn drain_messages(&mut self, key: &HalfKey) {
        let stream = self.streams.get_mut(key).unwrap();

        loop {
            if stream.data.len() < FIXED_HEADER_SIZE {
                break;
            }

            // Check for magic prefix
            if stream.data[0..2] != MAGIC {
                // Lost sync — try to find next magic
                if let Some(pos) = find_magic(&stream.data[1..]) {
                    stream.data.drain(..pos + 1);
                    continue;
                } else {
                    // No magic found, keep last byte (might be start of 'E')
                    let len = stream.data.len();
                    stream.data.drain(..len - 1);
                    break;
                }
            }

            // Read message length
            let msg_len = u32::from_be_bytes([
                stream.data[2], stream.data[3],
                stream.data[4], stream.data[5],
            ]) as usize;

            let total = 6 + msg_len;

            // Not enough data yet — wait for more packets
            if stream.data.len() < total {
                break;
            }

            // Try to parse the message
            match Message::parse(&stream.data[..total]) {
                Ok((message, _consumed)) => {
                    let ts = stream.first_ts;
                    self.messages.push(ParsedMessage {
                        message,
                        src: key.src,
                        dst: key.dst,
                        timestamp_us: ts,
                    });
                    stream.data.drain(..total);
                    stream.first_ts = 0; // will be set by next add_payload
                }
                Err(_) => {
                    // Parse failed — skip past this magic and try again
                    stream.data.drain(..2);
                }
            }
        }
    }

    /// Finalize: return all parsed messages.
    pub fn finish(self) -> Vec<ParsedMessage> {
        self.messages
    }
}

/// Find the position of "ES" magic in data.
fn find_magic(data: &[u8]) -> Option<usize> {
    data.windows(2).position(|w| w == MAGIC)
}
