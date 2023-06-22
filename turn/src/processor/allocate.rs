use anyhow::Result;
use bytes::BytesMut;
use crate::{
    SOFTWARE,
    StunClass,
};

use super::{
    Context,
    Response,
};

use std::{
    net::SocketAddr,
    sync::Arc,
};

use faster_stun::{
    MessageReader,
    MessageWriter,
    Method,
    Kind,
};

use faster_stun::attribute::{
    Error,
    ErrKind,
    ErrorCode,
    Realm,
    Nonce,
    ReqeestedTransport,
    XorMappedAddress,
    XorRelayedAddress,
    Lifetime,
    UserName,
    Software,
};

use faster_stun::attribute::ErrKind::{
    Unauthorized,
    ServerError,
};

/// return allocate error response
#[inline(always)]
fn reject<'a, 'b, 'c>(
    ctx: Context,
    m: MessageReader<'a, 'b>,
    w: &'c mut BytesMut,
    e: ErrKind,
) -> Result<Response<'c>> {
    let method = Method::Allocate(Kind::Error);
    let nonce = ctx.env.router.get_nonce(&ctx.addr);
    let mut pack = MessageWriter::extend(method, &m, w);
    pack.append::<ErrorCode>(Error::from(e));
    pack.append::<Realm>(&ctx.env.realm);
    pack.append::<Nonce>(&nonce);
    pack.flush(None)?;
    Ok(Some((w, StunClass::Message, None)))
}

/// return allocate ok response
///
/// NOTE: The use of randomized port assignments to avoid certain
/// types of attacks is described in [RFC6056].  It is RECOMMENDED
/// that a TURN server implement a randomized port assignment
/// algorithm from [RFC6056].  This is especially applicable to
/// servers that choose to pre-allocate a number of ports from the
/// underlying OS and then later assign them to allocations; for
/// example, a server may choose this technique to implement the
/// EVEN-PORT attribute.
#[inline(always)]
fn resolve<'a, 'b, 'c>(
    ctx: &Context,
    m: &MessageReader<'a, 'b>,
    p: &[u8; 16],
    port: u16,
    w: &'c mut BytesMut,
) -> Result<Response<'c>> {
    let method = Method::Allocate(Kind::Response);
    let alloc_addr = Arc::new(SocketAddr::new(ctx.env.external.ip(), port));
    let mut pack = MessageWriter::extend(method, m, w);
    pack.append::<XorRelayedAddress>(*alloc_addr.as_ref());
    pack.append::<XorMappedAddress>(ctx.addr);
    pack.append::<Lifetime>(600);
    pack.append::<Software>(SOFTWARE);
    pack.flush(Some(p))?;
    Ok(Some((w, StunClass::Message, None)))
}

/// process allocate request
///
/// [rfc8489](https://tools.ietf.org/html/rfc8489)
///
/// In all cases, the server SHOULD only allocate ports from the range
/// 49152 - 65535 (the Dynamic and/or Private Port range [PORT-NUMBERS]),
/// unless the TURN server application knows, through some means not
/// specified here, that other applications running on the same host as
/// the TURN server application will not be impacted by allocating ports
/// outside this range.  This condition can often be satisfied by running
/// the TURN server application on a dedicated machine and/or by
/// arranging that any other applications on the machine allocate ports
/// before the TURN server application starts.  In any case, the TURN
/// server SHOULD NOT allocate ports in the range 0 - 1023 (the Well-
/// Known Port range) to discourage clients from using TURN to run
/// standard services.
pub async fn process<'a, 'b, 'c>(
    ctx: Context,
    m: MessageReader<'a, 'b>,
    w: &'c mut BytesMut,
) -> Result<Response<'c>> {
    if m.get::<ReqeestedTransport>().is_none() {
        return reject(ctx, m, w, ServerError);
    }

    let u = match m.get::<UserName>() {
        None => return reject(ctx, m, w, Unauthorized),
        Some(u) => u,
    };

    let key = match ctx.env.router.get_key(ctx.env.index, &ctx.addr, u).await {
        None => return reject(ctx, m, w, Unauthorized),
        Some(p) => p,
    };

    let port = match ctx.env.router.alloc_port(&ctx.addr) {
        None => return reject(ctx, m, w, Unauthorized),
        Some(p) => p,
    };

    if m.integrity(&key).is_ok() {
        ctx.env.observer.allocated(&ctx.addr, u, port);
        resolve(&ctx, &m, &key, port, w)
    } else {
        reject(ctx, m, w, Unauthorized)
    }
}
