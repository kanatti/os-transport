use crate::protocol::vint::read_vint;

/// Read a length-prefixed UTF-8 string (VInt length + bytes).
/// Returns (string, bytes_consumed).
pub fn read_string(data: &[u8]) -> Result<(String, usize), &'static str> {
    let (len, len_bytes) = read_vint(data)?;
    let len = len as usize;
    let total = len_bytes + len;

    if data.len() < total {
        return Err("not enough data for string");
    }

    let s = std::str::from_utf8(&data[len_bytes..total])
        .map_err(|_| "invalid UTF-8 in string")?;

    Ok((s.to_string(), total))
}

/// Read an optional string (boolean prefix + string if true).
/// Returns (Option<String>, bytes_consumed).
pub fn read_optional_string(data: &[u8]) -> Result<(Option<String>, usize), &'static str> {
    if data.is_empty() {
        return Err("not enough data for optional string");
    }

    if data[0] == 0 {
        return Ok((None, 1));
    }

    let (s, consumed) = read_string(&data[1..])?;
    Ok((Some(s), 1 + consumed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_string_hello() {
        // "hello" = VInt(5) + UTF-8 bytes
        let data = [0x05, 0x68, 0x65, 0x6C, 0x6C, 0x6F];
        assert_eq!(read_string(&data), Ok(("hello".to_string(), 6)));
    }

    #[test]
    fn test_read_string_empty() {
        let data = [0x00];
        assert_eq!(read_string(&data), Ok(("".to_string(), 1)));
    }

    #[test]
    fn test_read_string_multibyte_utf8() {
        // "你好" = VInt(6) + 6 UTF-8 bytes
        let data = [0x06, 0xE4, 0xBD, 0xA0, 0xE5, 0xA5, 0xBD];
        assert_eq!(read_string(&data), Ok(("你好".to_string(), 7)));
    }

    #[test]
    fn test_read_string_truncated() {
        let data = [0x05, 0x68, 0x65]; // says 5 bytes but only 2 available
        assert_eq!(read_string(&data), Err("not enough data for string"));
    }

    #[test]
    fn test_read_optional_string_none() {
        let data = [0x00];
        assert_eq!(read_optional_string(&data), Ok((None, 1)));
    }

    #[test]
    fn test_read_optional_string_some() {
        let data = [0x01, 0x04, 0x74, 0x65, 0x73, 0x74]; // true + "test"
        assert_eq!(read_optional_string(&data), Ok((Some("test".to_string()), 6)));
    }
}
