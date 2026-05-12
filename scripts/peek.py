#!/usr/bin/env python3
"""Parse an OpenSearch transport pcap and show message headers."""
import struct
import sys

def find_messages(data):
    """Find all potential OS transport messages by scanning for 'ES' magic."""
    positions = []
    i = 0
    while i < len(data) - 23:
        idx = data.find(b'ES', i)
        if idx == -1:
            break
        msg_len = struct.unpack('>I', data[idx+2:idx+6])[0]
        if 10 < msg_len < 100000:
            positions.append(idx)
        i = idx + 1
    return positions

def parse_header(data, pos):
    """Parse a 23-byte fixed header at the given position."""
    header = data[pos:pos+23]
    msg_len = struct.unpack('>I', header[2:6])[0]
    req_id = struct.unpack('>Q', header[6:14])[0]
    status = header[14]
    version_raw = struct.unpack('>I', header[15:19])[0]
    var_header_size = struct.unpack('>I', header[19:23])[0]

    os_mask = 0x08000000
    is_opensearch = (version_raw & os_mask) != 0
    version_id = version_raw ^ os_mask if is_opensearch else version_raw
    major = version_id // 1000000
    minor = (version_id % 1000000) // 10000
    patch = (version_id % 10000) // 100

    return {
        "msg_len": msg_len,
        "req_id": req_id,
        "status": status,
        "is_response": (status & 0x01) != 0,
        "is_error": (status & 0x02) != 0,
        "is_compressed": (status & 0x04) != 0,
        "is_handshake": (status & 0x08) != 0,
        "version": f"{major}.{minor}.{patch}",
        "is_opensearch": is_opensearch,
        "version_raw": version_raw,
        "var_header_size": var_header_size,
    }

def find_action(data, pos, var_header_size):
    """Extract action name from variable header."""
    var_start = pos + 23
    var_data = data[var_start:var_start + min(var_header_size, 300)]
    strings = []
    j = 0
    while j < len(var_data):
        if 0x20 <= var_data[j] < 0x7f:
            s = ""
            while j < len(var_data) and 0x20 <= var_data[j] < 0x7f:
                s += chr(var_data[j])
                j += 1
            if ":" in s and len(s) > 5:
                while s and s[0].isdigit():
                    s = s[1:]
                strings.append(s)
        else:
            j += 1
    return strings[0] if strings else None

def main():
    pcap_file = sys.argv[1] if len(sys.argv) > 1 else "testdata/captures/basic.pcap"

    with open(pcap_file, 'rb') as f:
        data = f.read()

    positions = find_messages(data)
    print(f"Found {len(positions)} potential messages\n")

    for pos in positions[:20]:
        h = parse_header(data, pos)
        flags = []
        if h["is_response"]: flags.append("RSP")
        else: flags.append("REQ")
        if h["is_error"]: flags.append("ERR")
        if h["is_compressed"]: flags.append("COMPRESSED")
        if h["is_handshake"]: flags.append("HANDSHAKE")

        action = ""
        if not h["is_response"]:
            action = find_action(data, pos, h["var_header_size"]) or ""

        print(f"0x{pos:04X}  #{h['req_id']:<6d}  {' '.join(flags):<12s}  {h['msg_len']:>6d}B  {h['version']}  {action}")

if __name__ == "__main__":
    main()
