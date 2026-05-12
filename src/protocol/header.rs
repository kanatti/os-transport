use crate::protocol::status::Status;
use crate::protocol::version::Version;

/// Fixed header size in bytes.
pub const FIXED_HEADER_SIZE: usize = 23;

/// Magic prefix bytes.
pub const MAGIC: [u8; 2] = [0x45, 0x53]; // "ES"

/// Parsed fixed header (23 bytes).
#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub msg_len: u32,
    pub request_id: u64,
    pub status: Status,
    pub version: Version,
    pub var_header_size: u32,
}

impl Header {
    /// Parse a fixed header from a byte slice (must be at least 23 bytes).
    pub fn parse(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < FIXED_HEADER_SIZE {
            return Err("not enough data for fixed header");
        }

        if data[0..2] != MAGIC {
            return Err("invalid magic prefix");
        }

        let msg_len = u32::from_be_bytes([data[2], data[3], data[4], data[5]]);
        let request_id = u64::from_be_bytes([
            data[6], data[7], data[8], data[9],
            data[10], data[11], data[12], data[13],
        ]);
        let status = Status::from_byte(data[14]);
        let version_raw = u32::from_be_bytes([data[15], data[16], data[17], data[18]]);
        let version = Version::from_raw(version_raw);
        let var_header_size = u32::from_be_bytes([data[19], data[20], data[21], data[22]]);

        Ok(Header { msg_len, request_id, status, version, var_header_size })
    }

    /// Total message size on the wire (including the 6-byte prefix+length).
    pub fn total_size(&self) -> usize {
        6 + self.msg_len as usize
    }
}

impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "#{} {} {}B {}  var_hdr={}B",
            self.request_id, self.status, self.msg_len, self.version, self.var_header_size
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request() {
        // Construct a minimal valid header
        let mut data = [0u8; 23];
        data[0] = 0x45; // 'E'
        data[1] = 0x53; // 'S'
        // msg_len = 243
        data[2..6].copy_from_slice(&243u32.to_be_bytes());
        // request_id = 433
        data[6..14].copy_from_slice(&433u64.to_be_bytes());
        // status = 0x00 (request)
        data[14] = 0x00;
        // version = OS 3.7.0 (0x082ED893)
        data[15..19].copy_from_slice(&0x082ED893u32.to_be_bytes());
        // var_header_size = 56
        data[19..23].copy_from_slice(&56u32.to_be_bytes());

        let header = Header::parse(&data).unwrap();
        assert_eq!(header.msg_len, 243);
        assert_eq!(header.request_id, 433);
        assert!(!header.status.is_response);
        assert_eq!(header.version.major, 3);
        assert_eq!(header.version.minor, 7);
        assert_eq!(header.var_header_size, 56);
    }

    #[test]
    fn test_bad_magic() {
        let data = [0x00u8; 23];
        assert_eq!(Header::parse(&data), Err("invalid magic prefix"));
    }

    #[test]
    fn test_too_short() {
        let data = [0x45, 0x53, 0x00];
        assert_eq!(Header::parse(&data), Err("not enough data for fixed header"));
    }

    #[test]
    fn test_from_capture() {
        // First message from our basic.pcap capture (manually extracted)
        // ES 000000F3 00000000000001B1 00 082ED893 00000038
        let data: [u8; 23] = [
            0x45, 0x53,                         // magic
            0x00, 0x00, 0x00, 0xF3,             // len = 243
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xB1, // req_id = 433
            0x00,                               // status = REQ
            0x08, 0x2E, 0xD8, 0x93,             // version = OS 3.7.0
            0x00, 0x00, 0x00, 0x38,             // var_header_size = 56
        ];

        let header = Header::parse(&data).unwrap();
        assert_eq!(header.msg_len, 243);
        assert_eq!(header.request_id, 433);
        assert_eq!(format!("{}", header.version), "OS 3.7.0");
        assert_eq!(header.var_header_size, 56);
    }
}
