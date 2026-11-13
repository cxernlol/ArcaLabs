use crate::{PublishError, endpoint};
use bytes::Bytes;
use rml_rtmp::{
    handshake::{Handshake, HandshakeProcessResult, PeerType},
    sessions::{
        ClientSession, ClientSessionConfig, ClientSessionEvent, ClientSessionResult,
        PublishRequestType,
    },
    time::RtmpTimestamp,
};
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned, pki_types::ServerName};
use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    sync::Arc,
    time::Duration,
};

type TlsStream = StreamOwned<ClientConnection, TcpStream>;

/// RTMPS session. Call from a dedicated network worker; the RTMP session is order-sensitive.
pub struct RtmpPublisher {
    stream: TlsStream,
    session: ClientSession,
}

impl RtmpPublisher {
    /// Opens a TLS connection and completes RTMP publish negotiation. Invoke only after Go Live.
    pub fn connect(ingest_url: &str, stream_key: &str) -> Result<Self, PublishError> {
        let endpoint = endpoint(ingest_url)?;
        if stream_key.trim().is_empty() {
            return Err(PublishError::InvalidEndpoint("stream key is empty".into()));
        }
        let address = (endpoint.host.as_str(), endpoint.port)
            .to_socket_addrs()
            .map_err(|error| PublishError::Transport(error.to_string()))?
            .next()
            .ok_or_else(|| PublishError::Transport("host resolved to no addresses".into()))?;
        let socket = TcpStream::connect_timeout(&address, Duration::from_secs(10))
            .map_err(|error| PublishError::Transport(error.to_string()))?;
        socket
            .set_nodelay(true)
            .and_then(|_| socket.set_read_timeout(Some(Duration::from_secs(10))))
            .and_then(|_| socket.set_write_timeout(Some(Duration::from_secs(10))))
            .map_err(|error| PublishError::Transport(error.to_string()))?;

        let mut roots = RootCertStore::empty();
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let config = ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        let server_name = ServerName::try_from(endpoint.host.clone())
            .map_err(|error| PublishError::Transport(error.to_string()))?;
        let connection = ClientConnection::new(Arc::new(config), server_name)
            .map_err(|error| PublishError::Transport(error.to_string()))?;
        let mut stream = StreamOwned::new(connection, socket);

        let remaining = complete_rtmp_handshake(&mut stream)?;
        let mut config = ClientSessionConfig::new();
        config.tc_url = Some(endpoint.tc_url.clone());
        let (mut session, _) = ClientSession::new(config)
            .map_err(|error| PublishError::Protocol(error.to_string()))?;
        let connect = session
            .request_connection(endpoint.app.clone())
            .map_err(|error| PublishError::Protocol(error.to_string()))?;
        write_result(&mut stream, connect)?;
        let connected =
            wait_for_event(&mut stream, &mut session, remaining, |event| match event {
                ClientSessionEvent::ConnectionRequestAccepted => Some(Ok(())),
                ClientSessionEvent::ConnectionRequestRejected { description } => {
                    Some(Err(PublishError::Rejected(description)))
                }
                _ => None,
            })?;
        connected?;

        let publish = session
            .request_publishing(stream_key.to_owned(), PublishRequestType::Live)
            .map_err(|error| PublishError::Protocol(error.to_string()))?;
        write_result(&mut stream, publish)?;
        wait_for_event(&mut stream, &mut session, Vec::new(), |event| match event {
            ClientSessionEvent::PublishRequestAccepted => Some(Ok(())),
            ClientSessionEvent::ConnectionRequestRejected { description } => {
                Some(Err(PublishError::Rejected(description)))
            }
            _ => None,
        })??;

        stream
            .get_ref()
            .set_read_timeout(Some(Duration::from_millis(1)))
            .map_err(|error| PublishError::Transport(error.to_string()))?;
        Ok(Self { stream, session })
    }

    /// Sends an FLV AVC video-tag payload produced by the muxer, with a millisecond timestamp.
    pub fn publish_video_tag(
        &mut self,
        payload: Bytes,
        timestamp_ms: u32,
    ) -> Result<(), PublishError> {
        self.poll_server()?;
        let result = self
            .session
            .publish_video_data(payload, RtmpTimestamp::new(timestamp_ms), false)
            .map_err(|error| PublishError::Protocol(error.to_string()))?;
        write_result(&mut self.stream, result)
    }

    /// Sends an FLV AAC audio-tag payload produced by the muxer, with a millisecond timestamp.
    pub fn publish_audio_tag(
        &mut self,
        payload: Bytes,
        timestamp_ms: u32,
    ) -> Result<(), PublishError> {
        self.poll_server()?;
        let result = self
            .session
            .publish_audio_data(payload, RtmpTimestamp::new(timestamp_ms), false)
            .map_err(|error| PublishError::Protocol(error.to_string()))?;
        write_result(&mut self.stream, result)
    }

    pub fn stop(mut self) -> Result<(), PublishError> {
        for result in self
            .session
            .stop_publishing()
            .map_err(|error| PublishError::Protocol(error.to_string()))?
        {
            write_result(&mut self.stream, result)?;
        }
        self.stream
            .get_ref()
            .shutdown(std::net::Shutdown::Both)
            .map_err(|error| PublishError::Transport(error.to_string()))
    }

    fn poll_server(&mut self) -> Result<(), PublishError> {
        let mut buffer = [0; 16 * 1024];
        loop {
            match self.stream.read(&mut buffer) {
                Ok(0) => {
                    return Err(PublishError::Transport(
                        "ingest server closed the connection".into(),
                    ));
                }
                Ok(count) => {
                    let results = self
                        .session
                        .handle_input(&buffer[..count])
                        .map_err(|error| PublishError::Protocol(error.to_string()))?;
                    for result in results {
                        write_result(&mut self.stream, result)?;
                    }
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    return Ok(());
                }
                Err(error) => return Err(PublishError::Transport(error.to_string())),
            }
        }
    }
}

fn complete_rtmp_handshake(stream: &mut TlsStream) -> Result<Vec<u8>, PublishError> {
    let mut handshake = Handshake::new(PeerType::Client);
    let initial = handshake
        .generate_outbound_p0_and_p1()
        .map_err(|error| PublishError::Protocol(error.to_string()))?;
    stream
        .write_all(&initial)
        .map_err(|error| PublishError::Transport(error.to_string()))?;

    let mut buffer = [0; 4096];
    loop {
        let count = stream
            .read(&mut buffer)
            .map_err(|error| PublishError::Transport(error.to_string()))?;
        if count == 0 {
            return Err(PublishError::Transport(
                "ingest closed during RTMP handshake".into(),
            ));
        }
        match handshake
            .process_bytes(&buffer[..count])
            .map_err(|error| PublishError::Protocol(error.to_string()))?
        {
            HandshakeProcessResult::InProgress { response_bytes } => {
                stream
                    .write_all(&response_bytes)
                    .map_err(|error| PublishError::Transport(error.to_string()))?;
            }
            HandshakeProcessResult::Completed {
                response_bytes,
                remaining_bytes,
            } => {
                stream
                    .write_all(&response_bytes)
                    .map_err(|error| PublishError::Transport(error.to_string()))?;
                return Ok(remaining_bytes);
            }
        }
    }
}

fn wait_for_event(
    stream: &mut TlsStream,
    session: &mut ClientSession,
    mut input: Vec<u8>,
    mut event_result: impl FnMut(ClientSessionEvent) -> Option<Result<(), PublishError>>,
) -> Result<Result<(), PublishError>, PublishError> {
    let mut buffer = [0; 16 * 1024];
    loop {
        if input.is_empty() {
            let count = stream
                .read(&mut buffer)
                .map_err(|error| PublishError::Transport(error.to_string()))?;
            if count == 0 {
                return Err(PublishError::Transport(
                    "ingest closed during publish negotiation".into(),
                ));
            }
            input.extend_from_slice(&buffer[..count]);
        }
        let results = session
            .handle_input(&input)
            .map_err(|error| PublishError::Protocol(error.to_string()))?;
        input.clear();
        for result in results {
            match result {
                ClientSessionResult::OutboundResponse(packet) => stream
                    .write_all(&packet.bytes)
                    .map_err(|error| PublishError::Transport(error.to_string()))?,
                ClientSessionResult::RaisedEvent(event) => {
                    if let Some(result) = event_result(event) {
                        return Ok(result);
                    }
                }
                ClientSessionResult::UnhandleableMessageReceived(_) => {}
            }
        }
    }
}

fn write_result(stream: &mut TlsStream, result: ClientSessionResult) -> Result<(), PublishError> {
    match result {
        ClientSessionResult::OutboundResponse(packet) => stream
            .write_all(&packet.bytes)
            .map_err(|error| PublishError::Transport(error.to_string())),
        ClientSessionResult::RaisedEvent(_)
        | ClientSessionResult::UnhandleableMessageReceived(_) => Ok(()),
    }
}
