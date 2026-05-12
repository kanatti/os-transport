use crate::protocol::header::Header;
use crate::protocol::var_header::{parse_variable_header, VariableHeader};

/// A fully parsed OS transport message (envelope only, body is raw bytes).
#[derive(Debug, Clone)]
pub struct Message {
    pub header: Header,
    pub var_header: VariableHeader,
    pub body: Vec<u8>,
}

impl Message {
    /// Parse a complete message from a byte slice starting at the "ES" magic.
    /// Returns (Message, total bytes consumed).
    pub fn parse(data: &[u8]) -> Result<(Self, usize), &'static str> {
        let header = Header::parse(data)?;

        let total_size = 6 + header.msg_len as usize; // 2 magic + 4 len + msg_len
        if data.len() < total_size {
            return Err("not enough data for complete message");
        }

        // Variable header starts after fixed header (23 bytes)
        let var_start = 23;
        let var_end = var_start + header.var_header_size as usize;

        if var_end > total_size {
            return Err("variable header size exceeds message length");
        }

        let var_header = parse_variable_header(
            &data[var_start..var_end],
            header.status.is_response,
        )?;

        // Body is everything after variable header
        let body = data[var_end..total_size].to_vec();

        Ok((Message { header, var_header, body }, total_size))
    }

    /// Action name (convenience accessor).
    pub fn action(&self) -> Option<&str> {
        self.var_header.action.as_deref()
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:<6} {:12} {:>6}B  {}", 
            self.header.request_id,
            format!("{}", self.header.status),
            self.header.msg_len,
            self.header.version,
        )?;
        if let Some(action) = self.action() {
            write!(f, "  {}", action)?;
        }
        Ok(())
    }
}
