use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};

pub const PROTOCOL_SUFFIX: &str = "\r\n";
pub const PROTOCOL_PREFIX: &str = "PROXY";
pub const TCP4: &str = "TCP4";
pub const TCP6: &str = "TCP6";
pub const UNKNOWN: &str = "UNKNOWN";

/// The sperator of the header parts.
pub const SEPARATOR: char = ' ';

/// A text PROXY protocol header that borrows the input string.
///
/// ## Examples
/// ### From byte slice
/// ```rust
/// use ppp::v1::{Addresses, Header};
///
/// let header = "PROXY UNKNOWN\r\n";
///
/// assert_eq!(Header::try_from(header.as_bytes()), Ok(Header::new(header, Addresses::Unknown)));
/// ```
///
/// ### UNKNOWN
/// ```rust
/// use ppp::v1::{Addresses, Header, UNKNOWN};
///
/// let input = "PROXY UNKNOWN\r\nhello";
/// let header = Header::try_from(input).unwrap();
///
/// assert_eq!(header, Header::new("PROXY UNKNOWN\r\n", Addresses::Unknown));
/// assert_eq!(header.protocol(), UNKNOWN);
/// assert_eq!(header.addresses(), "");
/// ```
///
/// ### TCP4
/// ```rust
/// use std::net::Ipv4Addr;
/// use ppp::v1::{Header, Addresses, TCP4};
///
/// let input = "PROXY TCP4 127.0.1.2 192.168.1.101 80 443\r\n";
/// let header = Header::try_from(input).unwrap();
///
/// assert_eq!(header, Header::new(input, Addresses::new_tcp4(Ipv4Addr::new(127, 0, 1, 2), Ipv4Addr::new(192, 168, 1, 101), 80, 443)));
/// assert_eq!(header.protocol(), TCP4);
/// assert_eq!(header.addresses(), "127.0.1.2 192.168.1.101 80 443");
/// ```
///
/// ### TCP6
/// ```rust
/// use std::net::Ipv6Addr;
/// use ppp::v1::{Header, Addresses, TCP6};
///
/// let input = "PROXY TCP6 1234:5678:90ab:cdef:fedc:ba09:8765:4321 4321:8765:ba09:fedc:cdef:90ab:5678:1234 443 65535\r\n";
/// let header = Header::try_from(input).unwrap();
///
/// assert_eq!(
///     header,
///     Header::new(
///         input,
///         Addresses::new_tcp6(
///             Ipv6Addr::from([0x1234, 0x5678, 0x90AB, 0xCDEF, 0xFEDC, 0xBA09, 0x8765, 0x4321]),
///             Ipv6Addr::from([0x4321, 0x8765, 0xBA09, 0xFEDC, 0xCDEF, 0x90AB, 0x5678, 0x01234,]),
///             443,
///             65535
///         )
///     )
/// );
/// assert_eq!(header.protocol(), TCP6);
/// assert_eq!(header.addresses(), "1234:5678:90ab:cdef:fedc:ba09:8765:4321 4321:8765:ba09:fedc:cdef:90ab:5678:1234 443 65535");
/// ```
///
/// ### Invalid
/// ```rust
/// use ppp::v1::{Header, Addresses, ParseError};
///
/// assert_eq!(Err(ParseError::InvalidProtocol), "PROXY tcp4\r\n".parse::<Addresses>());
/// ```
#[derive(Debug, PartialEq)]
pub struct Header<'a> {
    pub header: &'a str,
    pub addresses: Addresses,
}

impl<'a> Header<'a> {
    /// Creates a new `Header` with the given addresses and a reference to the original input.
    pub fn new(header: &'a str, addresses: Addresses) -> Self {
        Header { header, addresses }
    }

    /// The protocol portion of this `Header`.
    pub fn protocol(&self) -> &str {
        match self.addresses {
            Addresses::Tcp4(..) => TCP4,
            Addresses::Tcp6(..) => TCP6,
            Addresses::Unknown => UNKNOWN,
        }
    }

    /// The source and destination addressses portion of this `Header`.
    pub fn addresses(&self) -> &'a str {
        let start = PROTOCOL_PREFIX.len() + SEPARATOR.len_utf8() + self.protocol().len();
        let end = self.header.len() - PROTOCOL_SUFFIX.len();
        let addresses = &self.header[start..end];

        if addresses.starts_with(SEPARATOR) {
            &addresses[SEPARATOR.len_utf8()..]
        } else {
            addresses
        }
    }
}

/// The source and destination of a header.
/// Includes IP (v4 or v6) addresses and TCP ports.
///
/// ## Examples
/// ### UNKNOWN
/// ```rust
/// use ppp::v1::Addresses;
///
/// assert_eq!(Ok(Addresses::Unknown), "PROXY UNKNOWN\r\n".parse());
/// ```
///
/// ### TCP4
/// ```rust
/// use std::net::Ipv4Addr;
/// use ppp::v1::Addresses;
///
/// assert_eq!(
///     Ok(Addresses::new_tcp4(Ipv4Addr::new(127, 0, 1, 2), Ipv4Addr::new(192, 168, 1, 101), 80, 443)),
///     "PROXY TCP4 127.0.1.2 192.168.1.101 80 443\r\n".parse()
/// );
/// ```
///
/// ### TCP6
/// ```rust
/// use std::net::Ipv6Addr;
/// use ppp::v1::Addresses;
///
/// assert_eq!(
///     Ok(Addresses::new_tcp6(
///         Ipv6Addr::from([0x1234, 0x5678, 0x90AB, 0xCDEF, 0xFEDC, 0xBA09, 0x8765, 0x4321]),
///         Ipv6Addr::from([0x4321, 0x8765, 0xBA09, 0xFEDC, 0xCDEF, 0x90AB, 0x5678, 0x01234,]),
///         443,
///         65535
///     )),
///     "PROXY TCP6 1234:5678:90ab:cdef:fedc:ba09:8765:4321 4321:8765:ba09:fedc:cdef:90ab:5678:1234 443 65535\r\n".parse()
/// );
/// ```
///
/// ### Invalid
/// ```rust
/// use ppp::v1::{Addresses, ParseError};
///
/// assert_eq!(Err(ParseError::InvalidProtocol), "PROXY tcp4\r\n".parse::<Addresses>());
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Addresses {
    Tcp4(Tcp4),
    Tcp6(Tcp6),
    Unknown,
}

impl Addresses {
    /// Create a new IPv4 TCP address.
    pub fn new_tcp4(
        source_address: Ipv4Addr,
        destination_address: Ipv4Addr,
        source_port: u16,
        destination_port: u16,
    ) -> Self {
        Addresses::Tcp4(Tcp4 {
            source_address,
            source_port,
            destination_address,
            destination_port,
        })
    }

    /// Create a new IPv6 TCP address.
    pub fn new_tcp6(
        source_address: Ipv6Addr,
        destination_address: Ipv6Addr,
        source_port: u16,
        destination_port: u16,
    ) -> Self {
        Addresses::Tcp6(Tcp6 {
            source_address,
            source_port,
            destination_address,
            destination_port,
        })
    }
}

impl Default for Addresses {
    fn default() -> Self {
        Addresses::Unknown
    }
}

impl<'a> fmt::Display for Header<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.header)
    }
}

/// The source and destination IPv4 addresses and TCP ports of a header.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tcp4 {
    pub source_address: Ipv4Addr,
    pub source_port: u16,
    pub destination_address: Ipv4Addr,
    pub destination_port: u16,
}

/// The source and destination IPv6 addresses and TCP ports of a header.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tcp6 {
    pub source_address: Ipv6Addr,
    pub source_port: u16,
    pub destination_address: Ipv6Addr,
    pub destination_port: u16,
}
