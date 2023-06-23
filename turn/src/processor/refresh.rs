use bytes::BytesMut;
use anyhow::Result;
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
    ErrKind::Unauthorized,
    ErrKind,
    Error,
    ErrorCode,
    Lifetime,
    UserName,
};

/// return refresh error response
#[inline(always)]
fn reject<'a, 'b, 'c>(
    m: MessageReader<'a, 'b>,
    w: &'c mut BytesMut,
    e: ErrKind,
) -> Result<Response<'c>> {
    let method = Method::Refresh(Kind::Error);
    let mut pack = MessageWriter::extend(method, &m, w);
    pack.append::<ErrorCode>(Error::from(e));
    pack.flush(None)?;
    Ok(Some((w, StunClass::Message, None)))
}

/// return refresh ok response
#[inline(always)]
pub fn resolve<'a, 'b, 'c>(
    m: &MessageReader<'a, 'b>,
    lifetime: u32,
    p: &[u8; 16],
    w: &'c mut BytesMut,
) -> Result<Response<'c>> {
    let method = Method::Refresh(Kind::Response);
    let mut pack = MessageWriter::extend(method, m, w);
    pack.append::<Lifetime>(lifetime);
    pack.flush(Some(p))?;
    Ok(Some((w, StunClass::Message, None)))
}

/// process refresh request
///
/// If the server receives a Refresh Request with a REQUESTED-ADDRESS-
/// FAMILY attribute and the attribute value does not match the address
/// family of the allocation, the server MUST reply with a 443 (Peer
/// Address Family Mismatch) Refresh error response.
///
/// The server computes a value called the "desired lifetime" as follows:
/// if the request contains a LIFETIME attribute and the attribute value
/// is zero, then the "desired lifetime" is zero.  Otherwise, if the
/// request contains a LIFETIME attribute, then the server computes the
/// minimum of the client's requested lifetime and the server's maximum
/// allowed lifetime.  If this computed value is greater than the default
/// lifetime, then the "desired lifetime" is the computed value.
/// Otherwise, the "desired lifetime" is the default lifetime.
///
/// Subsequent processing depends on the "desired lifetime" value:
///
/// * If the "desired lifetime" is zero, then the request succeeds and
/// the allocation is deleted.
///
/// * If the "desired lifetime" is non-zero, then the request succeeds
/// and the allocation's time-to-expiry is set to the "desired
/// lifetime".
///
/// If the request succeeds, then the server sends a success response
/// containing:
///
/// * A LIFETIME attribute containing the current value of the time-to-
/// expiry timer.
///
/// NOTE: A server need not do anything special to implement
/// idempotency of Refresh requests over UDP using the "stateless
/// stack approach".  Retransmitted Refresh requests with a non-
/// zero "desired lifetime" will simply refresh the allocation.  A
/// retransmitted Refresh request with a zero "desired lifetime"
/// will cause a 437 (Allocation Mismatch) response if the
/// allocation has already been deleted, but the client will treat
/// this as equivalent to a success response (see below).
pub async fn process<'a, 'b, 'c>(
    ctx: Context,
    m: MessageReader<'a, 'b>,
    w: &'c mut BytesMut,
) -> Result<Response<'c>> {
    let u = match m.get::<UserName>() {
        Some(u) => u,
        _ => return reject(m, w, Unauthorized),
    };

    let l = match m.get::<Lifetime>() {
        Some(l) => l,
        _ => 600,
    };

    let key = match ctx.env.router.get_key(ctx.env.index, &ctx.addr, u).await {
        None => return reject(m, w, Unauthorized),
        Some(a) => a,
    };

    if m.integrity(&key).is_err() {
        return reject(m, w, Unauthorized);
    }

    ctx.env.observer.refresh(&ctx.addr, u, l);
    ctx.env.router.refresh(&ctx.addr, l);
    resolve(&m, l, &key, w)
}
