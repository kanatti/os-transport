/// Decode a variable-length u32 (1-5 bytes).
/// Returns (value, bytes_consumed).
pub fn read_vint(data: &[u8]) -> Result<(u32, usize), &'static str> {
    if data.is_empty() {
        return Err("unexpected end of data");
    }

    let mut result: u32 = 0;
    let mut shift: u32 = 0;

    // Byte format: [C|d d d d d d d]
    //               │ └─── 7 data bits
    //               └───── continue: 1=more, 0=done
    // Bytes chain little-endian: byte0=bits[0:6], byte1=bits[7:13], ...
    //   e.g. 300 = [0xAC, 0x02]
    //     0xAC → [1|0101100]  bits 0-6  (low)
    //     0x02 → [0|0000010]  bits 7-13 (high)
    //     result = 0000010_0101100 = 300
    for i in 0..5 {
        if i >= data.len() {
            return Err("unexpected end of data");
        }

        let byte = data[i];
        result |= ((byte & 0x7F) as u32) << shift;

        if byte & 0x80 == 0 {
            return Ok((result, i + 1));
        }

        shift += 7;
    }

    Err("vint too long (more than 5 bytes)")
}

/// Read a variable-length encoded 64-bit integer.
/// Uses 1-10 bytes.
/// Returns (value, bytes_consumed).
pub fn read_vlong(data: &[u8]) -> Result<(u64, usize), &'static str> {
    if data.is_empty() {
        return Err("unexpected end of data");
    }

    let mut result: u64 = 0;
    let mut shift: u32 = 0;

    for i in 0..10 {
        if i >= data.len() {
            return Err("unexpected end of data");
        }

        let byte = data[i];
        result |= ((byte & 0x7F) as u64) << shift;

        if byte & 0x80 == 0 {
            return Ok((result, i + 1));
        }

        shift += 7;
    }

    Err("vlong too long (more than 10 bytes)")
}

/// Decode a zigzag-encoded signed i64.
/// Zigzag maps small magnitudes to small unsigned values regardless of sign:
///   0 → 0, 1 → -1, 2 → 1, 3 → -2, 4 → 2, 5 → -3, ...
/// Without zigzag, -1 would need 10 bytes (all bits set). With it, just 1 byte.
pub fn read_zlong(data: &[u8]) -> Result<(i64, usize), &'static str> {
    let (encoded, consumed) = read_vlong(data)?;
    let decoded = ((encoded >> 1) as i64) ^ -((encoded & 1) as i64);
    Ok((decoded, consumed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vint_single_byte() {
        assert_eq!(read_vint(&[0x00]), Ok((0, 1)));
        assert_eq!(read_vint(&[0x05]), Ok((5, 1)));
        assert_eq!(read_vint(&[0x7F]), Ok((127, 1)));
    }

    #[test]
    fn test_vint_two_bytes() {
        // 300 = 0b100101100 → [0xAC, 0x02]
        assert_eq!(read_vint(&[0xAC, 0x02]), Ok((300, 2)));
        // 128 = 0b10000000 → [0x80, 0x01]
        assert_eq!(read_vint(&[0x80, 0x01]), Ok((128, 2)));
    }

    #[test]
    fn test_vint_max() {
        // i32::MAX = 2147483647 → [0xFF, 0xFF, 0xFF, 0xFF, 0x07]
        assert_eq!(
            read_vint(&[0xFF, 0xFF, 0xFF, 0xFF, 0x07]),
            Ok((2147483647, 5))
        );
    }

    #[test]
    fn test_vint_empty() {
        assert_eq!(read_vint(&[]), Err("unexpected end of data"));
    }

    #[test]
    fn test_vint_extra_bytes_ignored() {
        // Should only consume 1 byte, ignore the rest
        assert_eq!(read_vint(&[0x05, 0xFF, 0xFF]), Ok((5, 1)));
    }

    #[test]
    fn test_zlong() {
        // 0 → 0
        assert_eq!(read_zlong(&[0x00]), Ok((0, 1)));
        // 1 → -1
        assert_eq!(read_zlong(&[0x01]), Ok((-1, 1)));
        // 2 → 1
        assert_eq!(read_zlong(&[0x02]), Ok((1, 1)));
        // 9 → -5
        assert_eq!(read_zlong(&[0x09]), Ok((-5, 1)));
        // 10 → 5
        assert_eq!(read_zlong(&[0x0A]), Ok((5, 1)));
    }
}
