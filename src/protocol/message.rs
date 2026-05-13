use crate::protocol::header::{Header, FIXED_HEADER_SIZE, MAGIC};
use crate::protocol::var_header::{VariableHeader, parse_variable_header};

/// Size of the message prefix before msg_len payload begins (magic + length field).
const PREFIX_SIZE: usize = MAGIC.len() + 4;

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

        let total_size = PREFIX_SIZE + header.msg_len as usize;
        if data.len() < total_size {
            return Err("not enough data for complete message");
        }

        // Variable header starts after fixed header
        let var_start = FIXED_HEADER_SIZE;
        let var_end = var_start + header.var_header_size as usize;

        if var_end > total_size {
            return Err("variable header size exceeds message length");
        }

        let var_header =
            parse_variable_header(&data[var_start..var_end], header.status.is_response())?;

        // Body is everything after variable header
        let body = data[var_end..total_size].to_vec();

        Ok((
            Message {
                header,
                var_header,
                body,
            },
            total_size,
        ))
    }

    /// Action name (convenience accessor).
    pub fn action(&self) -> Option<&str> {
        self.var_header.action.as_deref()
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{:<6} {:12} {:>6}B  {}",
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
