//! ## Session Traversal Utilities for NAT (STUN)
//! 
//! STUN is intended to be used in the context of one or more NAT
//! traversal solutions.  These solutions are known as "STUN Usages".
//! Each usage describes how STUN is utilized to achieve the NAT
//! traversal solution.  Typically, a usage indicates when STUN messages
//! get sent, which optional attributes to include, what server is used,
//! and what authentication mechanism is to be used.  Interactive
//! Connectivity Establishment (ICE) [RFC8445](https://tools.ietf.org/html/rfc8445) 
//! is one usage of STUN. SIP Outbound [RFC5626](https://tools.ietf.org/html/rfc5626) 
//! is another usage of STUN.  In some cases, a usage will require extensions to STUN.  
//! A STUN extension can be in the form of new methods, attributes, or error response codes. 
//! More information on STUN Usages can be found in 
//! [Section 13](https://tools.ietf.org/html/rfc8489#section-13).
//!
//! ### STUN Message Structure
//!
//! ```bash
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
//! ```bash
//! 0                   1                   2                   3
//! 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |         Type                  |            Length             |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |                         Value (variable)                ....
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! ```
//!

pub mod attribute;
pub mod util;
mod message;
mod channel;

use anyhow::Result;
use attribute::AttrKind;
use std::convert::TryFrom;
use num_enum::TryFromPrimitive;
pub use channel::ChannelData;
pub use message::*;

/// message type.
#[repr(u16)]
#[derive(TryFromPrimitive)]
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    BindingRequest              = 0x0001,
    BindingResponse             = 0x0101,
    BindingError                = 0x0111,
    AllocateRequest             = 0x0003,
    AllocateResponse            = 0x0103,
    AllocateError               = 0x0113,
    CreatePermissionRequest     = 0x0008,
    CreatePermissionResponse    = 0x0108,
    CreatePermissionError       = 0x0118,
    SendIndication              = 0x0016,
    DataIndication              = 0x0017,
    ChannelBindRequest          = 0x0009,
    ChannelBindResponse         = 0x0109,
    ChannelBindError            = 0x0119,
    RefreshRequest              = 0x0004,
    RefreshResponse             = 0x0104,
    RefreshError                = 0x0114,
}

/// stun message payload.
pub enum Payload<'a, 'b> {
    Message(MessageReader<'a, 'b>),
    ChannelData(ChannelData<'a>),
}

/// stun decoder.
pub struct Decoder<'a> {
    attrs: Vec<(AttrKind, &'a [u8])>
}

impl<'a> Decoder<'a> {
    pub fn new() -> Self {
        Self {
            attrs: Vec::with_capacity(10)
        }
    }

    /// # Unit Test
    ///
    /// ```
    /// use stun::*;
    /// use stun::attribute::*;
    /// 
    /// let buffer = [
    ///     0x00, 0x09, 0x00, 0x58, 0x21, 0x12, 0xa4, 0x42,
    ///     0x35, 0x6a, 0x52, 0x42, 0x33, 0x4c, 0x65, 0x68,
    ///     0x2b, 0x7a, 0x75, 0x52, 0x00, 0x0c, 0x00, 0x04,
    ///     0x40, 0x00, 0x00, 0x00, 0x00, 0x12, 0x00, 0x08,
    ///     0x00, 0x01, 0xe1, 0x10, 0x5e, 0x12, 0xa4, 0x43,
    ///     0x00, 0x06, 0x00, 0x03, 0x64, 0x65, 0x76, 0x00,
    ///     0x00, 0x14, 0x00, 0x09, 0x6c, 0x6f, 0x63, 0x61,
    ///     0x6c, 0x68, 0x6f, 0x73, 0x74, 0x00, 0x00, 0x00,
    ///     0x00, 0x15, 0x00, 0x10, 0x6c, 0x37, 0x7a, 0x38,
    ///     0x33, 0x6b, 0x6c, 0x36, 0x61, 0x35, 0x63, 0x73,
    ///     0x77, 0x74, 0x74, 0x34, 0x00, 0x08, 0x00, 0x14,
    ///     0xbd, 0xb8, 0xee, 0x7d, 0xc8, 0x9f, 0x85, 0x1b,
    ///     0x5f, 0x18, 0x9a, 0x7b, 0x84, 0x3a, 0xfd, 0x88,
    ///     0xde, 0x03, 0xc0, 0x34
    /// ];
    ///     
    /// let mut decoder = Decoder::new();
    /// let payload = decoder.decode(&buffer).unwrap();
    /// if let Payload::Message(reader) = payload {
    ///     assert!(reader.get::<ChannelNumber>().is_some())
    /// }
    /// ```
    pub fn decode(&mut self, buf: &'a [u8]) -> Result<Payload<'a, '_>> {
        assert!(buf.len() >= 4);
        if !self.attrs.is_empty() {
            self.attrs.clear();
        }

        Ok(if buf[0] >> 4 == 4 {
            Payload::ChannelData(ChannelData::try_from(buf)?)
        } else {
            Payload::Message(MessageReader::decode(buf, &mut self.attrs)?)
        })
    }
}
