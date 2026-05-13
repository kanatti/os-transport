use crate::protocol::string::read_string;
use crate::protocol::vint::read_vint;

/// Read a list of strings.
/// Wire: [vint:count] [string] × count
///
/// ```text
/// [02] [05 "hello"] [02 "ok"]  →  ["hello", "ok"]
///  │    └── string    └── string
///  └── count = 2
/// ```
pub fn read_string_list(data: &[u8]) -> Result<(Vec<String>, usize), &'static str> {
    let (count, mut offset) = read_vint(data)?;

    let mut items = Vec::new();
    for _ in 0..count {
        let (s, consumed) = read_string(&data[offset..])?;
        offset += consumed;
        items.push(s);
    }

    Ok((items, offset))
}

/// Read a string-to-string map.
/// Wire: [vint:count] [key, value] × count
///
/// ```text
/// [01] [04 "name"] [05 "alice"]  →  [("name", "alice")]
///  │    └── key      └── value
///  └── count = 1
/// ```
pub fn read_string_map(data: &[u8]) -> Result<(Vec<(String, String)>, usize), &'static str> {
    let (count, mut offset) = read_vint(data)?;

    let mut pairs = Vec::new();
    for _ in 0..count {
        let (key, consumed) = read_string(&data[offset..])?;
        offset += consumed;
        let (value, consumed) = read_string(&data[offset..])?;
        offset += consumed;
        pairs.push((key, value));
    }

    Ok((pairs, offset))
}

/// Read a string-to-string-set map (each key maps to a list of values).
/// Wire: [vint:count] [key, string_list] × count
///
/// ```text
/// [01] [04 "role"] [02] [05 "admin"] [04 "user"]
///  │    └── key     │    └── values
///  └── 1 entry      └── 2 values
///
/// →  [("role", ["admin", "user"])]
/// ```
pub fn read_string_set_map(
    data: &[u8],
) -> Result<(Vec<(String, Vec<String>)>, usize), &'static str> {
    let (count, mut offset) = read_vint(data)?;

    let mut entries = Vec::new();
    for _ in 0..count {
        let (key, consumed) = read_string(&data[offset..])?;
        offset += consumed;
        let (values, consumed) = read_string_list(&data[offset..])?;
        offset += consumed;
        entries.push((key, values));
    }

    Ok((entries, offset))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_string_list_empty() {
        let data = vec![0x00]; // count = 0
        let (items, consumed) = read_string_list(&data).unwrap();
        assert_eq!(items.len(), 0);
        assert_eq!(consumed, 1);
    }

    #[test]
    fn test_read_string_list() {
        let mut data = vec![0x02]; // count = 2
        // "hi"
        data.push(0x02);
        data.extend_from_slice(b"hi");
        // "ok"
        data.push(0x02);
        data.extend_from_slice(b"ok");

        let (items, consumed) = read_string_list(&data).unwrap();
        assert_eq!(items, vec!["hi", "ok"]);
        assert_eq!(consumed, data.len());
    }

    #[test]
    fn test_read_string_map() {
        let mut data = vec![0x01]; // count = 1
        // key: "k"
        data.push(0x01);
        data.push(b'k');
        // value: "v"
        data.push(0x01);
        data.push(b'v');

        let (pairs, consumed) = read_string_map(&data).unwrap();
        assert_eq!(pairs, vec![("k".to_string(), "v".to_string())]);
        assert_eq!(consumed, data.len());
    }

    #[test]
    fn test_read_string_set_map() {
        let mut data = vec![0x01]; // count = 1
        // key: "k"
        data.push(0x01);
        data.push(b'k');
        // values: ["a", "b"]
        data.push(0x02); // 2 values
        data.push(0x01);
        data.push(b'a');
        data.push(0x01);
        data.push(b'b');

        let (entries, consumed) = read_string_set_map(&data).unwrap();
        assert_eq!(
            entries,
            vec![("k".to_string(), vec!["a".to_string(), "b".to_string()])]
        );
        assert_eq!(consumed, data.len());
    }
}
