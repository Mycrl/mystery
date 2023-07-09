use bytes::BytesMut;
use faster_stun::attribute::{
    XorMappedAddress,
    MappedAddress,
    ResponseOrigin,
    ReqeestedTransport,
    Transport,
    ErrorCode,
    ErrKind,
    Realm,
    UserName,
    XorRelayedAddress,
    Lifetime,
    XorPeerAddress,
    ChannelNumber,
};

use faster_stun::{
    MessageWriter,
    Decoder,
    Payload,
    Method,
    Kind,
    MessageReader,
};

use rand::seq::SliceRandom;
use tokio::net::UdpSocket;
use turn_server::{
    config::{
        *,
        self,
    },
    server_main,
};

use std::net::{
    IpAddr,
    SocketAddr,
    Ipv4Addr,
};

use std::sync::Arc;
use std::collections::HashMap;

const BIND_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const BIND_ADDR: SocketAddr = SocketAddr::new(BIND_IP, 3478);
const USERNAME: &'static str = "user1";
const PASSWORD: &'static str = "test";
const REALM: &'static str = "local-test";

fn create_token() -> [u8; 12] {
    let mut rng = rand::thread_rng();
    let mut token = [0u8; 12];
    token.shuffle(&mut rng);
    token
}

fn get_message_from_payload<'a, 'b>(
    payload: Payload<'a, 'b>,
) -> MessageReader<'a, 'b> {
    if let Payload::Message(m) = payload {
        m
    } else {
        panic!("get message from payload failed!")
    }
}

#[tokio::test]
async fn integration_testing() {
    let mut auth = HashMap::new();
    auth.insert(USERNAME.to_string(), PASSWORD.to_string());

    // Because it is testing, it is not reasonable to start a separate process
    // to start turn-server, and the configuration file is not convenient to
    // pass, so turn-server is used as a library here, and the server is
    // started with a custom configuration.
    tokio::spawn(async move {
        server_main(Arc::new(Config {
            auth,
            controller: Controller::default(),
            hooks: Hooks::default(),
            log: Log::default(),
            turn: Turn {
                threads: 1,
                realm: REALM.to_string(),
                interfaces: vec![Interface {
                    transport: config::Transport::UDP,
                    bind: BIND_ADDR,
                    external: BIND_ADDR,
                }],
            },
        }))
        .await
        .unwrap();
    });

    // Create a udp connection and connect to the turn-server, and then start
    // the corresponding session process checks in sequence. It should be noted
    // that the order of request responses is relatively strict, and should not
    // be changed under normal circumstances.
    let socket = UdpSocket::bind(SocketAddr::new(BIND_IP, 0)).await.unwrap();
    socket.connect(BIND_ADDR).await.unwrap();
    binding_request(&socket).await;
    base_allocate_request(&socket).await;
    let port = allocate_request(&socket).await;
    create_permission_request(&socket, port).await;
    channel_bind_request(&socket, port).await;
    refresh_request(&socket).await;
}

/// binding request
///
/// [rfc8489](https://tools.ietf.org/html/rfc8489)
///
/// In the Binding request/response transaction, a Binding request is
/// sent from a STUN client to a STUN server.  When the Binding request
/// arrives at the STUN server, it may have passed through one or more
/// NATs between the STUN client and the STUN server (in Figure 1, there
/// are two such NATs).  As the Binding request message passes through a
/// NAT, the NAT will modify the source transport address (that is, the
/// source IP address and the source port) of the packet.  As a result,
/// the source transport address of the request received by the server
/// will be the public IP address and port created by the NAT closest to
/// the server.  This is called a "reflexive transport address".  The
/// STUN server copies that source transport address into an XOR-MAPPED-
/// ADDRESS attribute in the STUN Binding response and sends the Binding
/// response back to the STUN client.  As this packet passes back through
/// a NAT, the NAT will modify the destination transport address in the
/// IP header, but the transport address in the XOR-MAPPED-ADDRESS
/// attribute within the body of the STUN response will remain untouched.
/// In this way, the client can learn its reflexive transport address
/// allocated by the outermost NAT with respect to the STUN server.
async fn binding_request(socket: &UdpSocket) {
    let token = create_token();
    let mut buf = BytesMut::with_capacity(1500);
    let mut msg =
        MessageWriter::new(Method::Binding(Kind::Request), &token, &mut buf);

    msg.flush(None).unwrap();
    socket.send_to(&buf, BIND_ADDR).await.unwrap();

    let mut buf = [0u8; 1500];
    let mut decoder = Decoder::new();
    let size = socket.recv(&mut buf).await.unwrap();
    let ret = get_message_from_payload(decoder.decode(&buf[..size]).unwrap());

    assert_eq!(ret.method, Method::Binding(Kind::Response));
    assert_eq!(ret.token, &token);

    let value = ret.get::<XorMappedAddress>().unwrap();
    assert_eq!(value, socket.local_addr().unwrap());

    let value = ret.get::<MappedAddress>().unwrap();
    assert_eq!(value, socket.local_addr().unwrap());

    let value = ret.get::<ResponseOrigin>().unwrap();
    assert_eq!(value, BIND_ADDR);
}

/// allocate request
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
async fn base_allocate_request(socket: &UdpSocket) {
    let token = create_token();
    let mut buf = BytesMut::with_capacity(1500);
    let mut msg =
        MessageWriter::new(Method::Allocate(Kind::Request), &token, &mut buf);

    msg.append::<ReqeestedTransport>(Transport::UDP);
    msg.flush(None).unwrap();
    socket.send_to(&buf, BIND_ADDR).await.unwrap();

    let mut buf = [0u8; 1500];
    let mut decoder = Decoder::new();
    let size = socket.recv(&mut buf).await.unwrap();
    let ret = get_message_from_payload(decoder.decode(&buf[..size]).unwrap());

    assert_eq!(ret.method, Method::Allocate(Kind::Error));
    assert_eq!(ret.token, &token);

    let value = ret.get::<ErrorCode>().unwrap();
    assert_eq!(value.code, ErrKind::Unauthorized as u16);

    let value = ret.get::<Realm>().unwrap();
    assert_eq!(value, REALM);
}

/// allocate request
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
///
/// NOTE: The use of randomized port assignments to avoid certain
/// types of attacks is described in [RFC6056].  It is RECOMMENDED
/// that a TURN server implement a randomized port assignment
/// algorithm from [RFC6056].  This is especially applicable to
/// servers that choose to pre-allocate a number of ports from the
/// underlying OS and then later assign them to allocations; for
/// example, a server may choose this technique to implement the
/// EVEN-PORT attribute.
async fn allocate_request(socket: &UdpSocket) -> u16 {
    let token = create_token();
    let key = faster_stun::util::long_key(USERNAME, PASSWORD, REALM);
    let mut buf = BytesMut::with_capacity(1500);
    let mut msg =
        MessageWriter::new(Method::Allocate(Kind::Request), &token, &mut buf);

    msg.append::<ReqeestedTransport>(Transport::UDP);
    msg.append::<UserName>(USERNAME);
    msg.append::<Realm>(REALM);
    msg.flush(Some(&key)).unwrap();
    socket.send_to(&buf, BIND_ADDR).await.unwrap();

    let mut buf = [0u8; 1500];
    let mut decoder = Decoder::new();
    let size = socket.recv(&mut buf).await.unwrap();
    let ret = get_message_from_payload(decoder.decode(&buf[..size]).unwrap());

    assert_eq!(ret.method, Method::Allocate(Kind::Response));
    assert_eq!(ret.token, &token);
    ret.integrity(&key).unwrap();

    let relay = ret.get::<XorRelayedAddress>().unwrap();
    assert_eq!(relay.ip(), BIND_IP);

    let value = ret.get::<XorMappedAddress>().unwrap();
    assert_eq!(value, socket.local_addr().unwrap());

    let value = ret.get::<Lifetime>().unwrap();
    assert_eq!(value, 600);

    relay.port()
}

/// create permission request
///
/// [rfc8489](https://tools.ietf.org/html/rfc8489)
///
/// When the server receives the CreatePermission request, it processes
/// as per [Section 5](https://tools.ietf.org/html/rfc8656#section-5)
/// plus the specific rules mentioned here.
///
/// The message is checked for validity.  The CreatePermission request
/// MUST contain at least one XOR-PEER-ADDRESS attribute and MAY contain
/// multiple such attributes.  If no such attribute exists, or if any of
/// these attributes are invalid, then a 400 (Bad Request) error is
/// returned.  If the request is valid, but the server is unable to
/// satisfy the request due to some capacity limit or similar, then a 508
/// (Insufficient Capacity) error is returned.
///
/// If an XOR-PEER-ADDRESS attribute contains an address of an address
/// family that is not the same as that of a relayed transport address
/// for the allocation, the server MUST generate an error response with
/// the 443 (Peer Address Family Mismatch) response code.
///
/// The server MAY impose restrictions on the IP address allowed in the
/// XOR-PEER-ADDRESS attribute; if a value is not allowed, the server
/// rejects the request with a 403 (Forbidden) error.
///
/// If the message is valid and the server is capable of carrying out the
/// request, then the server installs or refreshes a permission for the
/// IP address contained in each XOR-PEER-ADDRESS attribute as described
/// in [Section 9](https://tools.ietf.org/html/rfc8656#section-9).  
/// The port portion of each attribute is ignored and may be any arbitrary
/// value.
///
/// The server then responds with a CreatePermission success response.
/// There are no mandatory attributes in the success response.
///
/// > NOTE: A server need not do anything special to implement
/// idempotency of CreatePermission requests over UDP using the
/// "stateless stack approach".  Retransmitted CreatePermission
/// requests will simply refresh the permissions.
async fn create_permission_request(socket: &UdpSocket, port: u16) {
    let token = create_token();
    let key = faster_stun::util::long_key(USERNAME, PASSWORD, REALM);
    let mut buf = BytesMut::with_capacity(1500);
    let mut msg = MessageWriter::new(
        Method::CreatePermission(Kind::Request),
        &token,
        &mut buf,
    );

    msg.append::<XorPeerAddress>(SocketAddr::new(BIND_IP, port));
    msg.append::<UserName>(USERNAME);
    msg.append::<Realm>(REALM);
    msg.flush(Some(&key)).unwrap();
    socket.send_to(&buf, BIND_ADDR).await.unwrap();

    let mut buf = [0u8; 1500];
    let mut decoder = Decoder::new();
    let size = socket.recv(&mut buf).await.unwrap();
    let ret = get_message_from_payload(decoder.decode(&buf[..size]).unwrap());

    assert_eq!(ret.method, Method::CreatePermission(Kind::Response));
    assert_eq!(ret.token, &token);
    ret.integrity(&key).unwrap();
}

/// channel binding request
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
async fn channel_bind_request(socket: &UdpSocket, port: u16) {
    let token = create_token();
    let key = faster_stun::util::long_key(USERNAME, PASSWORD, REALM);
    let mut buf = BytesMut::with_capacity(1500);
    let mut msg = MessageWriter::new(
        Method::ChannelBind(Kind::Request),
        &token,
        &mut buf,
    );

    msg.append::<ChannelNumber>(0x4000);
    msg.append::<XorPeerAddress>(SocketAddr::new(BIND_IP, port));
    msg.append::<UserName>(USERNAME);
    msg.append::<Realm>(REALM);
    msg.flush(Some(&key)).unwrap();
    socket.send_to(&buf, BIND_ADDR).await.unwrap();

    let mut buf = [0u8; 1500];
    let mut decoder = Decoder::new();
    let size = socket.recv(&mut buf).await.unwrap();
    let ret = get_message_from_payload(decoder.decode(&buf[..size]).unwrap());

    assert_eq!(ret.method, Method::ChannelBind(Kind::Response));
    assert_eq!(ret.token, &token);
    ret.integrity(&key).unwrap();
}

/// refresh request
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
async fn refresh_request(socket: &UdpSocket) {
    let token = create_token();
    let key = faster_stun::util::long_key(USERNAME, PASSWORD, REALM);
    let mut buf = BytesMut::with_capacity(1500);
    let mut msg =
        MessageWriter::new(Method::Refresh(Kind::Request), &token, &mut buf);

    msg.append::<Lifetime>(0);
    msg.append::<UserName>(USERNAME);
    msg.append::<Realm>(REALM);
    msg.flush(Some(&key)).unwrap();
    socket.send_to(&buf, BIND_ADDR).await.unwrap();

    let mut buf = [0u8; 1500];
    let mut decoder = Decoder::new();
    let size = socket.recv(&mut buf).await.unwrap();
    let ret = get_message_from_payload(decoder.decode(&buf[..size]).unwrap());

    assert_eq!(ret.method, Method::Refresh(Kind::Response));
    assert_eq!(ret.token, &token);
    ret.integrity(&key).unwrap();

    let value = ret.get::<Lifetime>().unwrap();
    assert_eq!(value, 0);
}
