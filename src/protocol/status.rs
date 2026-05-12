/// Decoded status flags from byte 14 of the fixed header.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Status {
    pub is_response: bool,
    pub is_error: bool,
    pub is_compressed: bool,
    pub is_handshake: bool,
}

impl Status {
    pub fn from_byte(byte: u8) -> Self {
        Status {
            is_response: (byte & 0x01) != 0,
            is_error: (byte & 0x02) != 0,
            is_compressed: (byte & 0x04) != 0,
            is_handshake: (byte & 0x08) != 0,
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = if self.is_response { "RSP" } else { "REQ" };
        write!(f, "{}", kind)?;
        if self.is_error { write!(f, " ERR")?; }
        if self.is_compressed { write!(f, " COMPRESSED")?; }
        if self.is_handshake { write!(f, " HANDSHAKE")?; }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_uncompressed() {
        let s = Status::from_byte(0x00);
        assert!(!s.is_response);
        assert!(!s.is_error);
        assert!(!s.is_compressed);
        assert!(!s.is_handshake);
    }

    #[test]
    fn test_response() {
        let s = Status::from_byte(0x01);
        assert!(s.is_response);
        assert!(!s.is_error);
    }

    #[test]
    fn test_response_error() {
        let s = Status::from_byte(0x03);
        assert!(s.is_response);
        assert!(s.is_error);
    }

    #[test]
    fn test_compressed_request() {
        let s = Status::from_byte(0x04);
        assert!(!s.is_response);
        assert!(s.is_compressed);
    }

    #[test]
    fn test_handshake_response() {
        let s = Status::from_byte(0x09);
        assert!(s.is_response);
        assert!(s.is_handshake);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Status::from_byte(0x00)), "REQ");
        assert_eq!(format!("{}", Status::from_byte(0x03)), "RSP ERR");
        assert_eq!(format!("{}", Status::from_byte(0x04)), "REQ COMPRESSED");
    }
}
