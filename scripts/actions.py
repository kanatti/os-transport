#!/usr/bin/env python3
"""Show unique actions and their counts from an OS transport pcap."""
import struct
import sys

def main():
    pcap_file = sys.argv[1] if len(sys.argv) > 1 else "testdata/captures/basic.pcap"

    with open(pcap_file, 'rb') as f:
        data = f.read()

    # Find messages
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

    # Collect actions (requests only)
    actions = {}
    for pos in positions:
        status = data[pos + 14]
        if status & 0x01:  # skip responses
            continue

        var_header_size = struct.unpack('>I', data[pos+19:pos+23])[0]
        var_start = pos + 23
        var_data = data[var_start:var_start + min(var_header_size, 300)]

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
                    actions[s] = actions.get(s, 0) + 1
            else:
                j += 1

    print(f"Actions from {pcap_file} ({len(positions)} messages total):\n")
    for action, count in sorted(actions.items(), key=lambda x: -x[1]):
        print(f"  {count:4d}  {action}")

if __name__ == "__main__":
    main()
