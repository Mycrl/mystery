//
//  turn.h
//  turn-lib
//
//  Created by Mr.Panda on 2023/12/16.
//

#ifndef LIB_TURN__H
#define LIB_TURN__H
#pragma once

#include <stdint.h>

#ifdef __cplusplus
#include <optional>
#include <stdexcept>
#include <string>
#include <vector>
#endif

typedef enum
{
    Msg,
    Channel,
} StunClass;

typedef struct
{
    uint8_t* data;
    size_t data_len;
    StunClass kind;
    char* relay;
    char* interface;
} Response;

typedef enum
{
    InvalidInput,
    UnsupportedIpFamily,
    ShaFailed,
    NotIntegrity,
    IntegrityFailed,
    NotCookie,
    UnknownMethod,
    FatalError,
    Utf8Error,
} StunError;

typedef union
{
    Response response;
    StunError error;
} Result;

typedef struct
{
    bool is_success;
    Result result;
} ProcessRet;

typedef void(*GetPasswordCallback)(void* ctx, char* password);
typedef void(*ProcessCallback)(void* ctx, ProcessRet* ret);

typedef struct
{
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
    void (*get_password)(char* addr,
                         char* name,
                         GetPasswordCallback callback,
                         void* callback_ctx,
                         void* ctx);
    
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
    void (*allocated)(char* addr, char* name, uint16_t port, void* ctx);
    
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
    void (*binding)(char* addr, void* ctx);
    
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
    void (*channel_bind)(char* addr, char* name, uint16_t channel, void* ctx);
    
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
    void (*create_permission)(char* addr, char* name, char* relay, void* ctx);
    
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
    void (*refresh)(char* addr, char* name, uint32_t time, void* ctx);
    
    /// session abort
    ///
    /// Triggered when the node leaves from the turn. Possible reasons: the node
    /// life cycle has expired, external active deletion, or active exit of the
    /// node.
    void (*abort)(char* addr, char* name, void* ctx);
} Observer;

typedef void* Service;
typedef void* Processor;

extern "C" Service crate_turn_service(char* realm,
                                      char** externals,
                                      size_t externals_len,
                                      Observer observer,
                                      void* ctx);

extern "C" void drop_turn_service(Service service);

extern "C" Processor get_processor(Service service,
                                   char* interface,
                                   char* external);

extern "C" void drop_processor(Processor processor);

extern "C" void process(Processor processor,
                        uint8_t * buf,
                        size_t buf_len,
                        char* addr,
                        ProcessCallback callback,
                        void* ctx);

extern "C" void drop_process_ret(ProcessRet * ret);

extern "C" const char* stun_err_into_str(StunError kind)
{
    switch (kind)
    {
        case StunError::InvalidInput:
            return ("InvalidInput");
            break;
        case StunError::UnsupportedIpFamily:
            return ("UnsupportedIpFamily");
            break;
        case StunError::ShaFailed:
            return ("ShaFailed");
            break;
        case StunError::NotIntegrity:
            return ("NotIntegrity");
            break;
        case StunError::IntegrityFailed:
            return ("IntegrityFailed");
            break;
        case StunError::NotCookie:
            return ("NotCookie");
            break;
        case StunError::UnknownMethod:
            return ("UnknownMethod");
            break;
        case StunError::FatalError:
            return ("FatalError");
            break;
        case StunError::Utf8Error:
            return ("Utf8Error");
            break;
        default:
            break;
    }
}

#ifdef __cplusplus
class TurnObserver
{
public:
    virtual void GetPassword(std::string& addr,
                             std::string& name,
                             std::function<void(std::optional<std::string>)> callback)
    {
        callback(std::nullopt);
    }

    virtual void Allocated(std::string& addr, std::string& name, uint16_t port)
    {
    }

    virtual void Binding(std::string& addr)
    {
    }

    virtual void ChannelBind(std::string& addr,
                             std::string& name,
                             uint16_t channel)
    {
    }

    virtual void CreatePermission(std::string& addr,
                                  std::string& name,
                                  std::string& relay)
    {
    }

    virtual void Refresh(std::string& addr, std::string& name, uint32_t time)
    {
    }

    virtual void Abort(std::string& addr, std::string& name)
    {
    }
};

namespace StaticObserver
{
    void get_password(char* addr,
                      char* name,
                      GetPasswordCallback callback,
                      void* callback_ctx,
                      void* ctx)
    {
        auto observer = (TurnObserver*)ctx;
        auto addr_ = std::move(std::string(addr));
        auto name_ = std::move(std::string(name));
        observer->GetPassword(addr_, name_, [&](std::optional<std::string> ret)
                              {
                                  callback(callback_ctx, 
                                           ret.has_value()
                                            ? const_cast<char*>(ret.value().c_str())
                                            : nullptr);
                              });
    }

    void allocated(char* addr, char* name, uint16_t port, void* ctx)
    {
        auto observer = (TurnObserver*)ctx;
        auto addr_ = std::move(std::string(addr));
        auto name_ = std::move(std::string(name));
        observer->Allocated(addr_, name_, port);
    }

    void binding(char* addr, void* ctx)
    {
        auto observer = (TurnObserver*)ctx;
        auto addr_ = std::move(std::string(addr));
        observer->Binding(addr_);
    }

    void channel_bind(char* addr, char* name, uint16_t channel, void* ctx)
    {
        auto observer = (TurnObserver*)ctx;
        auto addr_ = std::move(std::string(addr));
        auto name_ = std::move(std::string(name));
        observer->ChannelBind(addr_, name_, channel);
    }

    void create_permission(char* addr, char* name, char* relay, void* ctx)
    {
        auto observer = (TurnObserver*)ctx;
        auto addr_ = std::move(std::string(addr));
        auto name_ = std::move(std::string(name));
        auto relay_ = std::move(std::string(relay));
        observer->CreatePermission(addr_, name_, relay_);
    }

    void refresh(char* addr, char* name, uint32_t time, void* ctx)
    {
        auto observer = (TurnObserver*)ctx;
        auto addr_ = std::move(std::string(addr));
        auto name_ = std::move(std::string(name));
        observer->Refresh(addr_, name_, time);
    }

    void abort(char* addr, char* name, void* ctx)
    {
        auto observer = (TurnObserver*)ctx;
        auto addr_ = std::move(std::string(addr));
        auto name_ = std::move(std::string(name));
        observer->Abort(addr_, name_);
    }

    static Observer Objects = { get_password, allocated, binding, channel_bind,
                               create_permission, refresh, abort };
} // namespace StaticObserver

class TurnProcessor
{
public:
    class Results
    {
    public:
        ProcessRet* Ret = nullptr;

        Results(ProcessRet* ret) : Ret(ret)
        {
        }

        ~Results()
        {
            if (Ret != nullptr)
            {
                drop_process_ret(Ret);
                Ret = nullptr;
            }
        }
    };

    TurnProcessor(Processor processor) : _processor(processor)
    {
    }

    ~TurnProcessor()
    {
        drop_processor(_processor);
    }

    void Process(uint8_t* buf,
                 size_t buf_len,
                 std::string& addr,
                 std::function<void(std::shared_ptr<Results>)> callback)
    {
        process(_processor,
                buf,
                buf_len,
                const_cast<char*>(addr.c_str()),
                ProcessCallback,
                &callback);
    }

private:
    Processor _processor;

    static void ProcessCallback(void* ctx, ProcessRet* ret)
    {
        auto callback = (std::function<void(std::shared_ptr<Results>)>*)ctx;
        (*callback)(ret == nullptr ? nullptr : std::make_shared<Results>(ret));
    }
};

class TurnService
{
public:
    TurnService(std::string& realm, std::vector<std::string> externals,
                TurnObserver* observer)
    {
        char* externals_[20];
        for (size_t i = 0; i < externals.size(); i++)
        {
            externals_[i] = const_cast<char*>(externals[i].c_str());
        }

        _service = crate_turn_service(const_cast<char*>(realm.c_str()),
                                      externals_,
                                      externals.size(),
                                      StaticObserver::Objects,
                                      observer);
        if (_service == nullptr)
        {
            throw std::runtime_error("crate turn service is failed!");
        }
    }

    ~TurnService()
    {
        drop_turn_service(_service);
    }

    TurnProcessor* GetProcessor(std::string& interface, std::string& external)
    {
        Processor processor = get_processor(_service,
                                            const_cast<char*>(interface.c_str()),
                                            const_cast<char*>(external.c_str()));
        if (processor == nullptr)
        {
            return (nullptr);
        }

        return (new TurnProcessor(processor));
    }

private:
    Service _service;
};
#endif // __cplusplus

#endif // LIB_TURN__H