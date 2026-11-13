#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    #[error("Only rtmps:// ingest URLs are accepted")]
    InsecureTransport,
    #[error("RTMP publishing is not connected")]
    NotConnected,
    #[error("RTMP publishing failed: {0}")]
    Protocol(String),
    #[error("RTMPS connection failed: {0}")]
    Transport(String),
    #[error("RTMPS endpoint is invalid: {0}")]
    InvalidEndpoint(String),
    #[error("The ingest server rejected the connection: {0}")]
    Rejected(String),
}

pub fn validate_ingest_url(url: &str) -> Result<(), PublishError> {
    endpoint(url).map(|_| ())
}

mod client;
mod flv;
pub use client::RtmpPublisher;
pub use flv::{
    FlvMuxError, aac_frame, aac_sequence_header, avc_frame, avc_frame_from_annex_b,
    avc_sequence_header, avc_sequence_header_from_annex_b,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Endpoint {
    host: String,
    port: u16,
    app: String,
    tc_url: String,
}

fn endpoint(input: &str) -> Result<Endpoint, PublishError> {
    let url =
        url::Url::parse(input).map_err(|error| PublishError::InvalidEndpoint(error.to_string()))?;
    if url.scheme() != "rtmps" {
        return Err(PublishError::InsecureTransport);
    }
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(PublishError::InvalidEndpoint(
            "credentials, query strings, and fragments are not accepted".into(),
        ));
    }
    let host = url
        .host_str()
        .ok_or_else(|| PublishError::InvalidEndpoint("host is missing".into()))?
        .to_owned();
    let app = url
        .path_segments()
        .into_iter()
        .flatten()
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("/");
    if app.is_empty() {
        return Err(PublishError::InvalidEndpoint(
            "application path is missing".into(),
        ));
    }
    let port = url.port().unwrap_or(443);
    let tc_url = format!("rtmps://{host}:{port}/{app}");
    Ok(Endpoint {
        host,
        port,
        app,
        tc_url,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn publish_requires_tls() {
        assert!(super::validate_ingest_url("rtmp://example/live").is_err());
        assert!(super::validate_ingest_url("rtmps://example/live").is_ok());
        assert!(super::validate_ingest_url("rtmps://example").is_err());
        assert!(super::validate_ingest_url("rtmps://user:secret@example/live").is_err());
    }

    #[test]
    fn endpoint_extracts_application_and_default_tls_port() {
        let endpoint = super::endpoint("rtmps://ingest.example/live/app").unwrap();
        assert_eq!(endpoint.host, "ingest.example");
        assert_eq!(endpoint.port, 443);
        assert_eq!(endpoint.app, "live/app");
        assert_eq!(endpoint.tc_url, "rtmps://ingest.example:443/live/app");
    }
}
