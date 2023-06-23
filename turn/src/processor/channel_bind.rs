use anyhow::Result;
use bytes::BytesMut;
use crate::StunClass;
use super::{
    Context,
    Response,
};

use faster_stun::{
    Kind,
    Method,
    MessageReader,
    MessageWriter,
};

use faster_stun::attribute::{
    ErrKind,
    Error,
    ErrorCode,
    Realm,
    UserName,
    ChannelNumber,
    XorPeerAddress,
};

use faster_stun::attribute::ErrKind::{
    BadRequest,
    Unauthorized,
    InsufficientCapacity,
    Forbidden,
};

/// return channel binding error response
#[inline(always)]
fn reject<'a, 'b, 'c>(
    ctx: Context,
    m: MessageReader<'a, 'b>,
    w: &'c mut BytesMut,
    e: ErrKind,
) -> Result<Response<'c>> {
    let method = Method::ChannelBind(Kind::Error);
    let mut pack = MessageWriter::extend(method, &m, w);
    pack.append::<ErrorCode>(Error::from(e));
    pack.append::<Realm>(&ctx.env.realm);
    pack.flush(None)?;
    Ok(Some((w, StunClass::Message, None)))
}

/// return channel binding ok response
#[inline(always)]
fn resolve<'c>(
    m: &MessageReader,
    p: &[u8; 16],
    w: &'c mut BytesMut,
) -> Result<Response<'c>> {
    let method = Method::ChannelBind(Kind::Response);
    MessageWriter::extend(method, m, w).flush(Some(p))?;
    Ok(Some((w, StunClass::Message, None)))
}

/// process channel binding request
///
/// The server MAY impose restrictions on the IP address and port values
/// allowed in the XOR-PEER-ADDRESS attribute; if a value is not allowed,
/// the server rejects the request with a 403 (Forbidden) error.
///
/// If the request is valid, but the server is unable to fulfill the
/// request due to some capacity limit or similar, the server replies
/// with a 508 (Insufficient Capacity) error.
///
/// Otherwise, the server replies with a ChannelBind success response.
/// There are no required attributes in a successful ChannelBind
/// response.
///
/// If the server can satisfy the request, then the server creates or
/// refreshes the channel binding using the channel number in the
/// CHANNEL-NUMBER attribute and the transport address in the XOR-PEER-
/// ADDRESS attribute.  The server also installs or refreshes a
/// permission for the IP address in the XOR-PEER-ADDRESS attribute as
/// described in Section 9.
///
/// NOTE: A server need not do anything special to implement
/// idempotency of ChannelBind requests over UDP using the
/// "stateless stack approach".  Retransmitted ChannelBind requests
/// will simply refresh the channel binding and the corresponding
/// permission.  Furthermore, the client must wait 5 minutes before
/// binding a previously bound channel number or peer address to a
/// different channel, eliminating the possibility that the
/// transaction would initially fail but succeed on a
/// retransmission.
pub async fn process<'a, 'b, 'c>(
    ctx: Context,
    m: MessageReader<'a, 'b>,
    w: &'c mut BytesMut,
) -> Result<Response<'c>> {
    let peer = match m.get::<XorPeerAddress>() {
        None => return reject(ctx, m, w, BadRequest),
        Some(a) => a,
    };

    let c = match m.get::<ChannelNumber>() {
        None => return reject(ctx, m, w, BadRequest),
        Some(c) => c,
    };

    if ctx.env.external.ip() != peer.ip() {
        return reject(ctx, m, w, Forbidden);
    }

    let u = match m.get::<UserName>() {
        None => return reject(ctx, m, w, Unauthorized),
        Some(u) => u,
    };

    if !(0x4000..=0x7FFF).contains(&c) {
        return reject(ctx, m, w, BadRequest);
    }

    let key = match ctx.env.router.get_key(ctx.env.index, &ctx.addr, u).await {
        None => return reject(ctx, m, w, Unauthorized),
        Some(a) => a,
    };

    if m.integrity(&key).is_err() {
        return reject(ctx, m, w, Unauthorized);
    }

    if ctx
        .env
        .router
        .bind_channel(&ctx.addr, peer.port(), c)
        .is_none()
    {
        return reject(ctx, m, w, InsufficientCapacity);
    }

    ctx.env.observer.channel_bind(&ctx.addr, u, c);
    resolve(&m, &key, w)
}
