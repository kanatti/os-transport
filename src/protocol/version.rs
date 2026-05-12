/// OpenSearch version decoded from the wire format.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub is_opensearch: bool,
    pub raw: u32,
}

const OS_MASK: u32 = 0x08000000;

impl Version {
    /// Decode a version from the raw 4-byte wire value.
    pub fn from_raw(raw: u32) -> Self {
        let is_opensearch = (raw & OS_MASK) != 0;
        let id = if is_opensearch { raw ^ OS_MASK } else { raw };

        let major = (id / 1_000_000) as u8;
        let minor = ((id % 1_000_000) / 10_000) as u8;
        let patch = ((id % 10_000) / 100) as u8;

        Version { major, minor, patch, is_opensearch, raw }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = if self.is_opensearch { "OS" } else { "ES" };
        write!(f, "{} {}.{}.{}", prefix, self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opensearch_3_7_0() {
        // From our capture: raw=0x082ED893
        let v = Version::from_raw(0x082ED893);
        assert!(v.is_opensearch);
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, 7);
        assert_eq!(v.patch, 0);
        assert_eq!(format!("{}", v), "OS 3.7.0");
    }

    #[test]
    fn test_opensearch_2_17_0() {
        // 2170099 ^ 0x08000000 = 0x08211E73
        let raw = 2_170_099 ^ OS_MASK;
        let v = Version::from_raw(raw);
        assert!(v.is_opensearch);
        assert_eq!(v.major, 2);
        assert_eq!(v.minor, 17);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn test_elasticsearch_7_10_2() {
        // ES 7.10.2: 7_100_299 (no mask)
        let v = Version::from_raw(7_100_299);
        assert!(!v.is_opensearch);
        assert_eq!(v.major, 7);
        assert_eq!(v.minor, 10);
        assert_eq!(v.patch, 2);
        assert_eq!(format!("{}", v), "ES 7.10.2");
    }
}
