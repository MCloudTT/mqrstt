use std::io;

use async_channel::{RecvError, SendError};

use crate::{
    packets::{
        error::{DeserializeError, SerializeError},
        {Packet, PacketType},
        reason_codes::ConnAckReasonCode,
    },
    util::timeout::Timeout,
};

#[derive(Debug, Clone, thiserror::Error)]
pub enum MqttError {
    #[error("Missing Packet ID")]
    MissingPacketId,

    #[error("The incoming channel between network and handler is closed")]
    IncomingNetworkChannelClosed,

    #[error("The outgoing channel between handler and network is closed: {0}")]
    OutgoingNetworkChannelClosed(#[from] SendError<Packet>),

    #[error("Channel between client and handler closed")]
    ClientChannelClosed,

    #[error("Packet Id error, pkid: {0}")]
    PacketIdError(u16),

    #[error("Received unsolicited ack pkid: {0}")]
    Unsolicited(u16, PacketType),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ClientError {
    #[error("One of more of the internal handler channels are closed")]
    NoHandler,

    #[error("Internal network channel is closed")]
    NoNetwork,
}

/// Critical errors during eventloop polling
#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    // #[error("Mqtt state: {0}")]
    // MqttState(#[from] StateError),
    #[error("Connect timeout")]
    Timeout(#[from] Timeout),

    #[cfg(feature = "use-rustls")]
    #[error("TLS: {0}")]
    Tls(#[from] tls::Error),

    #[error("No incoming packet handler available: {0}")]
    NoIncomingPacketHandler(#[from] SendError<Packet>),

    #[error("No outgoing packet sender: {0}")]
    NoOutgoingPacketSender(#[from] RecvError),

    #[error("Deserialization error: {0}")]
    DeserializationError(#[from] DeserializeError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] SerializeError),

    #[error("I/O: {0}")]
    Io(#[from] io::Error),

    #[error("Connection refused, return code: {0:?}")]
    ConnectionRefused(ConnAckReasonCode),

    #[error("Expected ConnAck packet, received: {0:?}")]
    NotConnAck(Packet),

    #[error("Requests done")]
    RequestsDone,

    #[error("TLS Error")]
    TLS(#[from] TlsError),
}

#[derive(Debug, thiserror::Error)]
pub enum TlsError {
    #[error("There is no or an incorrect TLS configuration for the requested TLS connection")]
    NoTlsConfig,

    #[error("Io error")]
    TlsIoError(#[from] io::Error),

    #[error("Could no construct a valid root certificate")]
    NoValidRootCertInChain,

    #[error("Could not construct a valid private key")]
    NoValidPrivateKey,

    #[cfg(any(feature = "tokio-rustls", feature = "smol-rustls"))]
    #[error("{0}")]
    RustlsError(#[from] rustls::Error),

    #[cfg(any(feature = "tokio-rustls", feature = "smol-rustls"))]
    #[error("{0}")]
    RustlsInvalidDnsNameError(#[from] rustls::client::InvalidDnsNameError),
}
