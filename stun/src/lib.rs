//! ## Session Traversal Utilities for NAT (STUN)
//!
//! [RFC8445]: https://tools.ietf.org/html/rfc8445
//! [RFC5626]: https://tools.ietf.org/html/rfc5626
//! [Section 13]: https://tools.ietf.org/html/rfc8489#section-13
//!
//! ### STUN Message Structure
//!
//! ```text
//! 0                   1                   2                   3
//! 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |0 0|     STUN Message Type     |         Message Length        |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |                         Magic Cookie                          |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |                                                               |
//! |                     Transaction ID (96 bits)                  |
//! |                                                               |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! ```
//!
//! ### STUN Attributes
//!
//! ```text
//! 0                   1                   2                   3
//! 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |         Type                  |            Length             |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |                         Value (variable)                ....
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! ```
//!
//! STUN is intended to be used in the context of one or more NAT
//! traversal solutions.  These solutions are known as "STUN Usages".
//! Each usage describes how STUN is utilized to achieve the NAT
//! traversal solution.  Typically, a usage indicates when STUN messages
//! get sent, which optional attributes to include, what server is used,
//! and what authentication mechanism is to be used.  Interactive
//! Connectivity Establishment (ICE) [RFC8445] is one usage of STUN.
//! SIP Outbound [RFC5626] is another usage of STUN.  In some cases,
//! a usage will require extensions to STUN. A STUN extension can be
//! in the form of new methods, attributes, or error response codes.
//! More information on STUN Usages can be found in [Section 13].

pub mod attribute;
pub mod channel;
pub mod message;
pub mod util;

pub use self::{
    attribute::{AttrKind, Transport},
    channel::ChannelData,
    message::*,
};

use std::ops::Range;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StunError {
    #[error("InvalidInput")]
    InvalidInput,
    #[error("SummaryFailed")]
    SummaryFailed,
    #[error("NotIntegrity")]
    NotIntegrity,
    #[error("IntegrityFailed")]
    IntegrityFailed,
    #[error("NotCookie")]
    NotCookie,
    #[error("UnknownMethod")]
    UnknownMethod,
    #[error("FatalError")]
    FatalError,
    #[error("Utf8Error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("TryFromSliceError: {0}")]
    TryFromSliceError(#[from] std::array::TryFromSliceError),
}

/// STUN Methods Registry
///
/// [RFC5389]: https://datatracker.ietf.org/doc/html/rfc5389
/// [RFC8489]: https://datatracker.ietf.org/doc/html/rfc8489
/// [RFC8126]: https://datatracker.ietf.org/doc/html/rfc8126
/// [Section 5]: https://datatracker.ietf.org/doc/html/rfc8489#section-5
///
/// A STUN method is a hex number in the range 0x000-0x0FF.  The encoding
/// of a STUN method into a STUN message is described in [Section 5].
///
/// STUN methods in the range 0x000-0x07F are assigned by IETF Review
/// [RFC8126].  STUN methods in the range 0x080-0x0FF are assigned by
/// Expert Review [RFC8126].  The responsibility of the expert is to
/// verify that the selected codepoint(s) is not in use and that the
/// request is not for an abnormally large number of codepoints.
/// Technical review of the extension itself is outside the scope of the
/// designated expert responsibility.
///
/// IANA has updated the name for method 0x002 as described below as well
/// as updated the reference from [RFC5389] to [RFC8489] for the following
/// STUN methods:
///
/// 0x000: Reserved
/// 0x001: Binding
/// 0x002: Reserved; was SharedSecret prior to [RFC5389]
/// 0x003: Allocate
/// 0x004: Refresh
/// 0x006: Send
/// 0x007: Data
/// 0x008: CreatePermission
/// 0x009: ChannelBind
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum Kind {
    Request,
    Response,
    Error,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum Method {
    Binding(Kind),
    Allocate(Kind),
    CreatePermission(Kind),
    ChannelBind(Kind),
    Refresh(Kind),
    SendIndication,
    DataIndication,
}

impl Method {
    pub fn is_error(&self) -> bool {
        match self {
            Method::Binding(Kind::Error)
            | Method::Refresh(Kind::Error)
            | Method::Allocate(Kind::Error)
            | Method::CreatePermission(Kind::Error)
            | Method::ChannelBind(Kind::Error) => true,
            _ => false,
        }
    }
}

impl TryFrom<u16> for Method {
    type Error = StunError;

    /// # Test
    ///
    /// ```
    /// use std::convert::TryFrom;
    /// use mycrl_stun::*;
    ///
    /// assert_eq!(
    ///     Method::try_from(0x0001).unwrap(),
    ///     Method::Binding(Kind::Request)
    /// );
    /// assert_eq!(
    ///     Method::try_from(0x0101).unwrap(),
    ///     Method::Binding(Kind::Response)
    /// );
    /// assert_eq!(
    ///     Method::try_from(0x0111).unwrap(),
    ///     Method::Binding(Kind::Error)
    /// );
    /// assert_eq!(
    ///     Method::try_from(0x0003).unwrap(),
    ///     Method::Allocate(Kind::Request)
    /// );
    /// assert_eq!(
    ///     Method::try_from(0x0103).unwrap(),
    ///     Method::Allocate(Kind::Response)
    /// );
    /// assert_eq!(
    ///     Method::try_from(0x0113).unwrap(),
    ///     Method::Allocate(Kind::Error)
    /// );
    /// assert_eq!(
    ///     Method::try_from(0x0008).unwrap(),
    ///     Method::CreatePermission(Kind::Request)
    /// );
    /// assert_eq!(
    ///     Method::try_from(0x0108).unwrap(),
    ///     Method::CreatePermission(Kind::Response)
    /// );
    /// assert_eq!(
    ///     Method::try_from(0x0118).unwrap(),
    ///     Method::CreatePermission(Kind::Error)
    /// );
    /// assert_eq!(
    ///     Method::try_from(0x0009).unwrap(),
    ///     Method::ChannelBind(Kind::Request)
    /// );
    /// assert_eq!(
    ///     Method::try_from(0x0109).unwrap(),
    ///     Method::ChannelBind(Kind::Response)
    /// );
    /// assert_eq!(
    ///     Method::try_from(0x0119).unwrap(),
    ///     Method::ChannelBind(Kind::Error)
    /// );
    /// assert_eq!(
    ///     Method::try_from(0x0004).unwrap(),
    ///     Method::Refresh(Kind::Request)
    /// );
    /// assert_eq!(
    ///     Method::try_from(0x0104).unwrap(),
    ///     Method::Refresh(Kind::Response)
    /// );
    /// assert_eq!(
    ///     Method::try_from(0x0114).unwrap(),
    ///     Method::Refresh(Kind::Error)
    /// );
    /// assert_eq!(Method::try_from(0x0016).unwrap(), Method::SendIndication);
    /// assert_eq!(Method::try_from(0x0017).unwrap(), Method::DataIndication);
    /// ```
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            0x0001 => Self::Binding(Kind::Request),
            0x0101 => Self::Binding(Kind::Response),
            0x0111 => Self::Binding(Kind::Error),
            0x0003 => Self::Allocate(Kind::Request),
            0x0103 => Self::Allocate(Kind::Response),
            0x0113 => Self::Allocate(Kind::Error),
            0x0008 => Self::CreatePermission(Kind::Request),
            0x0108 => Self::CreatePermission(Kind::Response),
            0x0118 => Self::CreatePermission(Kind::Error),
            0x0009 => Self::ChannelBind(Kind::Request),
            0x0109 => Self::ChannelBind(Kind::Response),
            0x0119 => Self::ChannelBind(Kind::Error),
            0x0004 => Self::Refresh(Kind::Request),
            0x0104 => Self::Refresh(Kind::Response),
            0x0114 => Self::Refresh(Kind::Error),
            0x0016 => Self::SendIndication,
            0x0017 => Self::DataIndication,
            _ => return Err(StunError::UnknownMethod),
        })
    }
}

impl From<Method> for u16 {
    /// # Test
    ///
    /// ```
    /// use std::convert::Into;
    /// use mycrl_stun::*;
    ///
    /// assert_eq!(0x0001u16, Method::Binding(Kind::Request).into());
    /// assert_eq!(0x0101u16, Method::Binding(Kind::Response).into());
    /// assert_eq!(0x0111u16, Method::Binding(Kind::Error).into());
    /// assert_eq!(0x0003u16, Method::Allocate(Kind::Request).into());
    /// assert_eq!(0x0103u16, Method::Allocate(Kind::Response).into());
    /// assert_eq!(0x0113u16, Method::Allocate(Kind::Error).into());
    /// assert_eq!(0x0008u16, Method::CreatePermission(Kind::Request).into());
    /// assert_eq!(0x0108u16, Method::CreatePermission(Kind::Response).into());
    /// assert_eq!(0x0118u16, Method::CreatePermission(Kind::Error).into());
    /// assert_eq!(0x0009u16, Method::ChannelBind(Kind::Request).into());
    /// assert_eq!(0x0109u16, Method::ChannelBind(Kind::Response).into());
    /// assert_eq!(0x0119u16, Method::ChannelBind(Kind::Error).into());
    /// assert_eq!(0x0004u16, Method::Refresh(Kind::Request).into());
    /// assert_eq!(0x0104u16, Method::Refresh(Kind::Response).into());
    /// assert_eq!(0x0114u16, Method::Refresh(Kind::Error).into());
    /// assert_eq!(0x0016u16, Method::SendIndication.into());
    /// assert_eq!(0x0017u16, Method::DataIndication.into());
    /// ```
    fn from(val: Method) -> Self {
        match val {
            Method::Binding(Kind::Request) => 0x0001,
            Method::Binding(Kind::Response) => 0x0101,
            Method::Binding(Kind::Error) => 0x0111,
            Method::Allocate(Kind::Request) => 0x0003,
            Method::Allocate(Kind::Response) => 0x0103,
            Method::Allocate(Kind::Error) => 0x0113,
            Method::CreatePermission(Kind::Request) => 0x0008,
            Method::CreatePermission(Kind::Response) => 0x0108,
            Method::CreatePermission(Kind::Error) => 0x0118,
            Method::ChannelBind(Kind::Request) => 0x0009,
            Method::ChannelBind(Kind::Response) => 0x0109,
            Method::ChannelBind(Kind::Error) => 0x0119,
            Method::Refresh(Kind::Request) => 0x0004,
            Method::Refresh(Kind::Response) => 0x0104,
            Method::Refresh(Kind::Error) => 0x0114,
            Method::SendIndication => 0x0016,
            Method::DataIndication => 0x0017,
        }
    }
}

#[derive(Debug)]
pub enum Payload<'a> {
    Message(MessageReader<'a>),
    ChannelData(ChannelData<'a>),
}

/// A cache of the list of attributes, this is for internal use only.
#[derive(Debug)]
pub struct Attributes(Vec<(AttrKind, Range<usize>)>);

impl Default for Attributes {
    fn default() -> Self {
        Self(Vec::with_capacity(20))
    }
}

impl Attributes {
    /// Adds an attribute to the list.
    pub fn append(&mut self, kind: AttrKind, range: Range<usize>) {
        self.0.push((kind, range));
    }

    /// Gets an attribute from the list.
    ///
    /// Note: This function will only look for the first matching property in
    /// the list and return it.
    pub fn get(&self, kind: &AttrKind) -> Option<Range<usize>> {
        self.0
            .iter()
            .find(|(k, _)| k == kind)
            .map(|(_, v)| v.clone())
    }

    /// Gets all the values of an attribute from a list.
    ///
    /// Normally a stun message can have multiple attributes with the same name,
    /// and this function will all the values of the current attribute.
    pub fn get_all<'a>(&'a self, kind: &'a AttrKind) -> impl Iterator<Item = &'a Range<usize>> {
        self.0
            .iter()
            .filter(move |(k, _)| k == kind)
            .map(|(_, v)| v)
            .into_iter()
    }

    pub fn clear(&mut self) {
        if !self.0.is_empty() {
            self.0.clear();
        }
    }
}

#[derive(Default)]
pub struct Decoder(Attributes);

impl Decoder {
    /// # Test
    ///
    /// ```
    /// use mycrl_stun::attribute::*;
    /// use mycrl_stun::*;
    ///
    /// let buffer = [
    ///     0x00, 0x01, 0x00, 0x4c, 0x21, 0x12, 0xa4, 0x42, 0x71, 0x66, 0x46, 0x31,
    ///     0x2b, 0x59, 0x79, 0x65, 0x56, 0x69, 0x32, 0x72, 0x00, 0x06, 0x00, 0x09,
    ///     0x55, 0x43, 0x74, 0x39, 0x3a, 0x56, 0x2f, 0x2b, 0x2f, 0x00, 0x00, 0x00,
    ///     0xc0, 0x57, 0x00, 0x04, 0x00, 0x00, 0x03, 0xe7, 0x80, 0x29, 0x00, 0x08,
    ///     0x22, 0x49, 0xda, 0x28, 0x2c, 0x6f, 0x2e, 0xdb, 0x00, 0x24, 0x00, 0x04,
    ///     0x6e, 0x00, 0x28, 0xff, 0x00, 0x08, 0x00, 0x14, 0x19, 0x58, 0xda, 0x38,
    ///     0xed, 0x1e, 0xdd, 0xc8, 0x6b, 0x8e, 0x22, 0x63, 0x3a, 0x22, 0x63, 0x97,
    ///     0xcf, 0xf5, 0xde, 0x82, 0x80, 0x28, 0x00, 0x04, 0x56, 0xf7, 0xa3, 0xed,
    /// ];
    ///
    /// let mut decoder = Decoder::default();
    /// let payload = decoder.decode(&buffer).unwrap();
    /// if let Payload::Message(reader) = payload {
    ///     assert!(reader.get::<UserName>().is_some())
    /// }
    /// ```
    pub fn decode<'a>(&'a mut self, bytes: &'a [u8]) -> Result<Payload<'a>, StunError> {
        assert!(bytes.len() >= 4);

        let flag = bytes[0] >> 6;
        if flag > 3 {
            return Err(StunError::InvalidInput);
        }

        Ok(if flag == 0 {
            self.0.clear();

            Payload::Message(MessageReader::decode(bytes, &mut self.0)?)
        } else {
            Payload::ChannelData(ChannelData::try_from(bytes)?)
        })
    }

    /// # Test
    ///
    /// ```
    /// use mycrl_stun::attribute::*;
    /// use mycrl_stun::*;
    ///
    /// let buffer = [
    ///     0x00, 0x01, 0x00, 0x4c, 0x21, 0x12, 0xa4, 0x42, 0x71, 0x66, 0x46, 0x31,
    ///     0x2b, 0x59, 0x79, 0x65, 0x56, 0x69, 0x32, 0x72, 0x00, 0x06, 0x00, 0x09,
    ///     0x55, 0x43, 0x74, 0x39, 0x3a, 0x56, 0x2f, 0x2b, 0x2f, 0x00, 0x00, 0x00,
    ///     0xc0, 0x57, 0x00, 0x04, 0x00, 0x00, 0x03, 0xe7, 0x80, 0x29, 0x00, 0x08,
    ///     0x22, 0x49, 0xda, 0x28, 0x2c, 0x6f, 0x2e, 0xdb, 0x00, 0x24, 0x00, 0x04,
    ///     0x6e, 0x00, 0x28, 0xff, 0x00, 0x08, 0x00, 0x14, 0x19, 0x58, 0xda, 0x38,
    ///     0xed, 0x1e, 0xdd, 0xc8, 0x6b, 0x8e, 0x22, 0x63, 0x3a, 0x22, 0x63, 0x97,
    ///     0xcf, 0xf5, 0xde, 0x82, 0x80, 0x28, 0x00, 0x04, 0x56, 0xf7, 0xa3, 0xed,
    /// ];
    ///
    /// let size = Decoder::message_size(&buffer, false).unwrap();
    /// assert_eq!(size, 96);
    /// ```
    pub fn message_size(bytes: &[u8], is_tcp: bool) -> Result<usize, StunError> {
        let flag = bytes[0] >> 6;
        if flag > 3 {
            return Err(StunError::InvalidInput);
        }

        Ok(if flag == 0 {
            MessageReader::message_size(bytes)?
        } else {
            ChannelData::message_size(bytes, is_tcp)?
        })
    }
}
