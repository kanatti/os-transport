use etherparse::NetSlice;
use etherparse::SlicedPacket;
use etherparse::TransportSlice;
use pcap_parser::traits::PcapReaderIterator;
use pcap_parser::*;
use std::fs::File;
use std::path::Path;

use crate::capture::reassembly::{Endpoint, ParsedMessage, TcpReassembler};

/// Read a pcap file and return all parsed OS transport messages.
pub fn read_pcap(path: &Path) -> Result<Vec<ParsedMessage>, String> {
    let file = File::open(path).map_err(|e| format!("failed to open file: {}", e))?;
    let mut reader = LegacyPcapReader::new(65536, file)
        .map_err(|e| format!("failed to create pcap reader: {:?}", e))?;

    let mut reassembler = TcpReassembler::new();
    let mut link_type = Linktype(0);

    loop {
        match reader.next() {
            Ok((offset, block)) => {
                match block {
                    PcapBlockOwned::LegacyHeader(header) => {
                        link_type = header.network;
                    }
                    PcapBlockOwned::Legacy(packet) => {
                        let ts_us = (packet.ts_sec as u64) * 1_000_000 + (packet.ts_usec as u64);
                        process_packet(&mut reassembler, packet.data, link_type, ts_us);
                    }
                    _ => {}
                }
                reader.consume(offset);
            }
            Err(PcapError::Eof) => break,
            Err(PcapError::Incomplete(_)) => {
                reader
                    .refill()
                    .map_err(|e| format!("refill error: {:?}", e))?;
            }
            Err(e) => return Err(format!("pcap error: {:?}", e)),
        }
    }

    Ok(reassembler.finish())
}

fn process_packet(reassembler: &mut TcpReassembler, data: &[u8], link_type: Linktype, ts_us: u64) {
    let parsed = match link_type {
        Linktype::NULL => {
            if data.len() <= 4 {
                return;
            }
            SlicedPacket::from_ip(&data[4..])
        }
        Linktype::ETHERNET => SlicedPacket::from_ethernet(data),
        Linktype::LINUX_SLL => SlicedPacket::from_linux_sll(data),
        _ => return,
    };

    let parsed = match parsed {
        Ok(p) => p,
        Err(_) => return,
    };

    // Extract src/dst IP
    let (src_addr, dst_addr) = match &parsed.net {
        Some(NetSlice::Ipv4(ipv4)) => (ipv4.header().source(), ipv4.header().destination()),
        _ => return,
    };

    // Extract TCP info
    let (src_port, dst_port, tcp_payload) = match &parsed.transport {
        Some(TransportSlice::Tcp(tcp)) => {
            (tcp.source_port(), tcp.destination_port(), tcp.payload())
        }
        _ => return,
    };

    let src = Endpoint {
        addr: src_addr,
        port: src_port,
    };
    let dst = Endpoint {
        addr: dst_addr,
        port: dst_port,
    };

    reassembler.add_payload(src, dst, tcp_payload, ts_us);
}
