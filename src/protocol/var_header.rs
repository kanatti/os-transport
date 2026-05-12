use crate::protocol::string::read_string;
use crate::protocol::vint::read_vint;

/// Parsed variable header contents.
#[derive(Debug, Clone, PartialEq)]
pub struct VariableHeader {
    /// Thread context request headers (key-value pairs).
    pub request_headers: Vec<(String, String)>,
    /// Thread context response headers (key -> set of values).
    pub response_headers: Vec<(String, Vec<String>)>,
    /// Action name (only present for requests).
    pub action: Option<String>,
}

/// Parse the variable header from a byte slice.
/// For requests: thread context + features + action name.
/// For responses: thread context + features.
pub fn parse_variable_header(data: &[u8], is_response: bool) -> Result<VariableHeader, &'static str> {
    let mut offset = 0;

    // Read request headers: VInt count, then key-value string pairs
    let (count, consumed) = read_vint(&data[offset..])?;
    offset += consumed;

    let mut request_headers = Vec::new();
    for _ in 0..count {
        let (key, consumed) = read_string(&data[offset..])?;
        offset += consumed;
        let (value, consumed) = read_string(&data[offset..])?;
        offset += consumed;
        request_headers.push((key, value));
    }

    // Read response headers: VInt count, then key -> string collection
    let (count, consumed) = read_vint(&data[offset..])?;
    offset += consumed;

    let mut response_headers = Vec::new();
    for _ in 0..count {
        let (key, consumed) = read_string(&data[offset..])?;
        offset += consumed;

        // Value is a string collection: VInt count + strings
        let (val_count, consumed) = read_vint(&data[offset..])?;
        offset += consumed;

        let mut values = Vec::new();
        for _ in 0..val_count {
            let (val, consumed) = read_string(&data[offset..])?;
            offset += consumed;
            values.push(val);
        }
        response_headers.push((key, values));
    }

    // Action name (requests only)
    let action = if !is_response {
        // Skip features (string collection for request)
        let (feature_count, consumed) = read_vint(&data[offset..])?;
        offset += consumed;
        for _ in 0..feature_count {
            let (_, consumed) = read_string(&data[offset..])?;
            offset += consumed;
        }

        // Action name
        let (action, _consumed) = read_string(&data[offset..])?;
        Some(action)
    } else {
        None
    };

    Ok(VariableHeader { request_headers, response_headers, action })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_headers_with_action() {
        // 0 request headers, 0 response headers, 0 features, action "cluster:monitor/health"
        let action_str = b"cluster:monitor/health";
        let action_len = action_str.len() as u8;

        let mut data = vec![
            0x00, // 0 request headers
            0x00, // 0 response headers
            0x00, // 0 features
            action_len, // action string length
        ];
        data.extend_from_slice(action_str);

        let vh = parse_variable_header(&data, false).unwrap();
        assert_eq!(vh.request_headers.len(), 0);
        assert_eq!(vh.response_headers.len(), 0);
        assert_eq!(vh.action, Some("cluster:monitor/health".to_string()));
    }

    #[test]
    fn test_with_request_headers() {
        let mut data = vec![
            0x01, // 1 request header
        ];
        // key: "X-Opaque-Id"
        let key = b"X-Opaque-Id";
        data.push(key.len() as u8);
        data.extend_from_slice(key);
        // value: "req-1"
        let val = b"req-1";
        data.push(val.len() as u8);
        data.extend_from_slice(val);
        // 0 response headers, 0 features
        data.push(0x00);
        data.push(0x00);
        // action
        let action = b"cluster:monitor/health";
        data.push(action.len() as u8);
        data.extend_from_slice(action);

        let vh = parse_variable_header(&data, false).unwrap();
        assert_eq!(vh.request_headers, vec![("X-Opaque-Id".to_string(), "req-1".to_string())]);
        assert_eq!(vh.action, Some("cluster:monitor/health".to_string()));
    }

    #[test]
    fn test_response_no_action() {
        // Response: just headers, no action
        let data = vec![
            0x00, // 0 request headers
            0x00, // 0 response headers
        ];

        let vh = parse_variable_header(&data, true).unwrap();
        assert_eq!(vh.action, None);
    }
}
