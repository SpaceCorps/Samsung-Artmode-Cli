//! Samsung Smart TV Art Mode client implementation over WebSocket and TCP.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, ClientConnection, DigitallySignedStruct, SignatureScheme, StreamOwned};
use serde_json::{Map, Value, json};
use tungstenite::{Message, WebSocket};

use crate::error::{Error, Result};

const ART_APP_ENDPOINT: &str = "com.samsung.art-app";
const APP_NAME_BASE64: &str = "U2Ftc3VuZ0FydENsaQ=="; // "SamsungArtCli" in base64

#[derive(Debug)]
struct AcceptAnyCert;

impl ServerCertVerifier for AcceptAnyCert {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        rustls::crypto::ring::default_provider().signature_verification_algorithms.supported_schemes()
    }
}

pub fn make_insecure_client_config() -> Arc<ClientConfig> {
    let config = ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
        .with_safe_default_protocol_versions()
        .unwrap()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(AcceptAnyCert))
        .with_no_client_auth();
    Arc::new(config)
}

pub enum TvStream {
    Plain(TcpStream),
    Tls(Box<StreamOwned<ClientConnection, TcpStream>>),
}

impl Read for TvStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            TvStream::Plain(s) => s.read(buf),
            TvStream::Tls(s) => s.read(buf),
        }
    }
}

impl Write for TvStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            TvStream::Plain(s) => s.write(buf),
            TvStream::Tls(s) => s.write(buf),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            TvStream::Plain(s) => s.flush(),
            TvStream::Tls(s) => s.flush(),
        }
    }
}

pub struct SamsungTvClient {
    ws: WebSocket<TvStream>,
    verbose: bool,
}

impl SamsungTvClient {
    /// Connects to a Samsung TV. If a token is provided, uses port 8002 with TLS.
    /// If no token is provided, uses port 8001 without TLS.
    pub fn connect(host: &str, token: Option<&str>, verbose: bool) -> Result<Self> {
        let (url, is_tls, target_addr) = parse_connection_target(host, token)?;

        if verbose {
            eprintln!("Connecting to TV at {url}...");
        }

        let tcp = TcpStream::connect(&target_addr).map_err(|e| {
            Error::network(format!("Could not connect to TV at {target_addr}: {e}"))
                .detail(format!("Check that the TV is powered on, connected to the network, and IP is correct. (Underlying error: {e})"))
                .fix(format!("samsung-artmode pair {host}"))
        })?;

        tcp.set_read_timeout(Some(Duration::from_secs(15)))?;
        tcp.set_write_timeout(Some(Duration::from_secs(15)))?;

        let stream = if is_tls {
            let config = make_insecure_client_config();
            let host_part = host.split(':').next().unwrap_or(host);
            let server_name = ServerName::try_from(host_part.to_string())
                .unwrap_or_else(|_| ServerName::try_from("samsung-tv".to_string()).unwrap());
            let conn = ClientConnection::new(config, server_name)
                .map_err(|e| Error::network(format!("TLS handshake initialization failed: {e}")))?;
            TvStream::Tls(Box::new(StreamOwned::new(conn, tcp)))
        } else {
            TvStream::Plain(tcp)
        };

        let (ws, response) = tungstenite::client(&url, stream).map_err(|e| {
            Error::network(format!("WebSocket handshake with TV failed: {e}"))
                .detail(format!("Handshake failed for {url}. The TV may have rejected the connection."))
                .fix("Check your TV and select 'Allow' when prompted, or run: samsung-artmode pair")
        })?;

        if verbose {
            eprintln!("Connected: HTTP status {}", response.status());
        }

        Ok(SamsungTvClient { ws, verbose })
    }

    /// Performs the pairing handshake on port 8001, prompts the user, and waits for auth token.
    pub fn pair(host: &str, verbose: bool, timeout: Duration) -> Result<String> {
        let mut client = Self::connect(host, None, verbose)?;
        let start = Instant::now();

        if verbose {
            eprintln!("Waiting for auth token from TV (timeout: {}s)...", timeout.as_secs());
        }

        while start.elapsed() < timeout {
            let msg = client.receive_raw()?;
            if verbose {
                eprintln!("WS << {msg}");
            }

            if let Ok(val) = serde_json::from_str::<Value>(&msg)
                && let Some(token) = val.get("data").and_then(|d| d.get("token")).and_then(Value::as_str)
            {
                return Ok(token.to_string());
            }
        }

        Err(Error::auth("No token received from TV. The connection may have timed out or been denied.")
            .detail("Make sure to press 'Allow' on the TV screen when the permission prompt appears.")
            .fix(format!("samsung-artmode pair {host}")))
    }

    /// Sends an Art Mode request over WebSocket and waits for the matching response.
    pub fn send_art_request(&mut self, request: &str, extra: Option<Map<String, Value>>) -> Result<Value> {
        let req_id = generate_id();
        let mut payload = Map::new();
        payload.insert("request".into(), Value::String(request.into()));
        payload.insert("id".into(), Value::String(req_id));

        if let Some(extra) = extra {
            for (k, v) in extra {
                payload.insert(k, v);
            }
        }

        let data_json =
            serde_json::to_string(&payload).map_err(|e| Error::other(format!("Failed to serialize payload: {e}")))?;

        let envelope = json!({
            "method": "ms.channel.emit",
            "params": {
                "event": "art_app_request",
                "to": "host",
                "data": data_json
            }
        });

        let envelope_str =
            serde_json::to_string(&envelope).map_err(|e| Error::other(format!("Failed to serialize envelope: {e}")))?;

        if self.verbose {
            eprintln!("WS >> {envelope_str}");
        }

        self.ws.send(Message::Text(envelope_str))?;

        self.wait_for_response(request, Duration::from_secs(15))
    }

    /// Uploads an image file to the TV and returns the assigned `content_id`.
    pub fn upload_image(&mut self, file_path: &Path, matte_id: Option<&str>) -> Result<String> {
        if !file_path.is_file() {
            return Err(Error::invalid(format!("File not found: {}", file_path.display())));
        }

        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        let file_type = match ext.as_str() {
            "jpg" | "jpeg" => "jpeg",
            "png" => "png",
            _ => {
                return Err(Error::invalid(format!(
                    "Unsupported image format: '{ext}'. Supported formats are PNG and JPEG."
                )));
            }
        };

        let file_bytes = std::fs::read(file_path)?;
        let request_id = generate_id();
        let conn_id: u32 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u32 % 900_000 + 100_000)
            .unwrap_or(123456);

        let now = now_image_date();

        let mut send_payload = Map::new();
        send_payload.insert("request".into(), json!("send_image"));
        send_payload.insert("file_type".into(), json!(file_type));
        send_payload.insert("file_size".into(), json!(file_bytes.len()));
        send_payload.insert("id".into(), json!(generate_id()));
        send_payload.insert("request_id".into(), json!(request_id));
        send_payload.insert("matte_id".into(), json!(matte_id.unwrap_or("none")));
        send_payload.insert("portrait_matte_id".into(), json!("none"));
        send_payload.insert("image_date".into(), json!(now));
        send_payload.insert(
            "conn_info".into(),
            json!({
                "d2d_mode": "socket",
                "connection_id": conn_id,
                "id": generate_id()
            }),
        );

        let data_json = serde_json::to_string(&send_payload)
            .map_err(|e| Error::other(format!("Failed to serialize upload payload: {e}")))?;

        let envelope = json!({
            "method": "ms.channel.emit",
            "params": {
                "event": "art_app_request",
                "to": "host",
                "data": data_json
            }
        });

        let envelope_str =
            serde_json::to_string(&envelope).map_err(|e| Error::other(format!("Failed to serialize envelope: {e}")))?;

        if self.verbose {
            eprintln!("WS >> {envelope_str}");
        }

        self.ws.send(Message::Text(envelope_str))?;

        // Phase 1: Wait for ready_to_use
        let conn_info = self.wait_for_upload_ready(Duration::from_secs(30))?;
        let ip = conn_info
            .get("ip")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::network("Missing 'ip' in ready_to_use connection info"))?;
        let port = conn_info
            .get("port")
            .and_then(Value::as_u64)
            .ok_or_else(|| Error::network("Missing 'port' in ready_to_use connection info"))? as u16;
        let secured = conn_info.get("secured").and_then(Value::as_bool).unwrap_or(false);
        let sec_key = conn_info.get("key").or_else(|| conn_info.get("secKey")).and_then(Value::as_str).unwrap_or("");

        if self.verbose {
            eprintln!("TCP binary uploading to {ip}:{port} (secured={secured}, size={})", file_bytes.len());
        }

        // Phase 2: TCP binary transfer
        self.upload_via_tcp(ip, port, secured, sec_key, file_type, &file_bytes)?;

        // Phase 3: Wait for image_added
        let content_id = self.wait_for_image_added(Duration::from_secs(30))?;
        Ok(content_id)
    }

    fn upload_via_tcp(
        &self,
        ip: &str,
        port: u16,
        secured: bool,
        sec_key: &str,
        file_type: &str,
        file_bytes: &[u8],
    ) -> Result<()> {
        let tcp = TcpStream::connect((ip, port))
            .map_err(|e| Error::network(format!("Failed to connect to TV binary upload socket at {ip}:{port}: {e}")))?;
        tcp.set_write_timeout(Some(Duration::from_secs(30)))?;

        let header = json!({
            "num": 0,
            "total": 1,
            "fileLength": file_bytes.len(),
            "fileName": "upload",
            "fileType": file_type,
            "secKey": sec_key,
            "version": "0.0.1"
        });

        let header_str = serde_json::to_string(&header)
            .map_err(|e| Error::other(format!("Failed to serialize binary header: {e}")))?;
        let header_bytes = header_str.as_bytes();
        let len_bytes = (header_bytes.len() as u32).to_be_bytes();

        if secured {
            let config = make_insecure_client_config();
            let server_name = ServerName::try_from(ip.to_string())
                .unwrap_or_else(|_| ServerName::try_from("samsung-tv".to_string()).unwrap());
            let conn = ClientConnection::new(config, server_name)
                .map_err(|e| Error::network(format!("TLS handshake failed for binary upload: {e}")))?;
            let mut stream = StreamOwned::new(conn, tcp);
            stream.write_all(&len_bytes)?;
            stream.write_all(header_bytes)?;

            const CHUNK_SIZE: usize = 65536;
            for chunk in file_bytes.chunks(CHUNK_SIZE) {
                stream.write_all(chunk)?;
            }
            stream.flush()?;
        } else {
            let mut stream = tcp;
            stream.write_all(&len_bytes)?;
            stream.write_all(header_bytes)?;

            const CHUNK_SIZE: usize = 65536;
            for chunk in file_bytes.chunks(CHUNK_SIZE) {
                stream.write_all(chunk)?;
            }
            stream.flush()?;
        }

        Ok(())
    }

    fn wait_for_response(&mut self, request: &str, timeout: Duration) -> Result<Value> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            let raw = self.receive_raw()?;
            if self.verbose {
                eprintln!("WS << {raw}");
            }

            if let Ok(doc) = serde_json::from_str::<Value>(&raw)
                && let Some(data) = extract_data(&doc)
            {
                if let Some(req) = data.get("request").and_then(Value::as_str)
                    && req == request
                {
                    return Ok(data);
                }
                if let Some(evt) = data.get("event").and_then(Value::as_str)
                    && evt == request
                {
                    return Ok(data);
                }
            }
        }

        Err(Error::network(format!("Timed out waiting for response to '{request}' from TV."))
            .detail("The TV did not reply within the 15-second timeout window.")
            .fix("Check that the TV is powered on, connected, and in Art Mode."))
    }

    fn wait_for_upload_ready(&mut self, timeout: Duration) -> Result<Value> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            let raw = self.receive_raw()?;
            if self.verbose {
                eprintln!("WS << {raw}");
            }

            if let Ok(doc) = serde_json::from_str::<Value>(&raw)
                && let Some(data) = extract_data(&doc)
            {
                let is_ready = data.get("event").and_then(Value::as_str) == Some("ready_to_use")
                    || data.get("request").and_then(Value::as_str) == Some("ready_to_use");

                if is_ready && let Some(conn_info) = data.get("conn_info") {
                    return Ok(conn_info.clone());
                }
            }
        }

        Err(Error::network("Timed out waiting for upload ready signal from TV.")
            .detail("The TV did not return 'ready_to_use' connection info."))
    }

    fn wait_for_image_added(&mut self, timeout: Duration) -> Result<String> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            let raw = self.receive_raw()?;
            if self.verbose {
                eprintln!("WS << {raw}");
            }

            if let Ok(doc) = serde_json::from_str::<Value>(&raw)
                && let Some(data) = extract_data(&doc)
            {
                let is_added = data.get("event").and_then(Value::as_str) == Some("image_added")
                    || data.get("request").and_then(Value::as_str) == Some("image_added");

                if is_added && let Some(cid) = data.get("content_id").and_then(Value::as_str) {
                    return Ok(cid.to_string());
                }
            }
        }

        Err(Error::network("Timed out waiting for image upload confirmation from TV.")
            .detail("The TV accepted binary chunks but did not confirm 'image_added'."))
    }

    fn receive_raw(&mut self) -> Result<String> {
        loop {
            let msg = self.ws.read().map_err(Error::from)?;
            match msg {
                Message::Text(s) => return Ok(s.to_string()),
                Message::Binary(b) => return Ok(String::from_utf8_lossy(&b).into_owned()),
                Message::Ping(payload) => {
                    let _ = self.ws.send(Message::Pong(payload));
                }
                Message::Pong(_) => {}
                Message::Close(_) => {
                    return Err(Error::network("WebSocket connection closed by TV."));
                }
                Message::Frame(_) => {}
            }
        }
    }

    // High-level Art Mode commands

    pub fn get_artmode_status(&mut self) -> Result<Value> {
        self.send_art_request("get_artmode_status", None)
    }

    pub fn get_content_list(&mut self, category: Option<&str>) -> Result<Value> {
        let mut extra = Map::new();
        if let Some(cat) = category {
            extra.insert("category".into(), json!(cat));
        }
        self.send_art_request("get_content_list", Some(extra))
    }

    pub fn get_current_artwork(&mut self) -> Result<Value> {
        self.send_art_request("get_current_artwork", None)
    }

    pub fn select_image(&mut self, content_id: &str, show: bool) -> Result<Value> {
        let mut extra = Map::new();
        extra.insert("content_id".into(), json!(content_id));
        extra.insert("show".into(), json!(show));
        self.send_art_request("select_image", Some(extra))
    }

    pub fn delete_image_list(&mut self, content_id: &str) -> Result<Value> {
        let mut extra = Map::new();
        extra.insert("content_id_list".into(), json!([{"content_id": content_id}]));
        self.send_art_request("delete_image_list", Some(extra))
    }

    pub fn change_favorite(&mut self, content_id: &str, remove: bool) -> Result<Value> {
        let mut extra = Map::new();
        extra.insert("content_id".into(), json!(content_id));
        extra.insert("status".into(), json!(if remove { "off" } else { "on" }));
        self.send_art_request("change_favorite", Some(extra))
    }

    pub fn get_brightness(&mut self) -> Result<Value> {
        self.send_art_request("get_brightness", None)
    }

    pub fn set_brightness(&mut self, value: u8) -> Result<Value> {
        let mut extra = Map::new();
        extra.insert("value".into(), json!(value));
        self.send_art_request("set_brightness", Some(extra))
    }

    pub fn get_matte_list(&mut self) -> Result<Value> {
        self.send_art_request("get_matte_list", None)
    }

    pub fn change_matte(&mut self, content_id: &str, matte_id: &str) -> Result<Value> {
        let mut extra = Map::new();
        extra.insert("content_id".into(), json!(content_id));
        extra.insert("matte_id".into(), json!(matte_id));
        extra.insert("portrait_matte_id".into(), json!("none"));
        self.send_art_request("change_matte", Some(extra))
    }

    pub fn set_slideshow(&mut self, off: bool, interval: Option<u32>, category: &str, shuffle: bool) -> Result<Value> {
        let mut extra = Map::new();
        if off {
            extra.insert("value".into(), json!("off"));
        } else {
            let iv = interval.unwrap_or(10);
            extra.insert("value".into(), json!(iv.to_string()));
            extra.insert("category_id".into(), json!(category));
            extra.insert("type".into(), json!(if shuffle { "shuffleslideshow" } else { "slideshow" }));
        }
        self.send_art_request("set_slideshow_status", Some(extra))
    }
}

fn parse_connection_target(host: &str, token: Option<&str>) -> Result<(String, bool, String)> {
    if host.starts_with("ws://") || host.starts_with("wss://") {
        let is_tls = host.starts_with("wss://");
        let parsed: tungstenite::http::Uri =
            host.parse().map_err(|e| Error::invalid(format!("Invalid URL: {host} ({e})")))?;
        let authority = parsed.authority().ok_or_else(|| Error::invalid("Missing host/authority in URL"))?;
        let host_name = authority.host();
        let port = authority.port_u16().unwrap_or(if is_tls { 8002 } else { 8001 });
        let target_addr = format!("{host_name}:{port}");

        let url = if let Some(tok) = token {
            if host.contains('?') {
                format!("{host}&name={APP_NAME_BASE64}&token={tok}")
            } else {
                format!("{host}/api/v2/channels/{ART_APP_ENDPOINT}?name={APP_NAME_BASE64}&token={tok}")
            }
        } else if host.contains('?') {
            format!("{host}&name={APP_NAME_BASE64}")
        } else {
            format!("{host}/api/v2/channels/{ART_APP_ENDPOINT}?name={APP_NAME_BASE64}")
        };

        return Ok((url, is_tls, target_addr));
    }

    let is_tls = token.is_some();
    let port = if is_tls { 8002 } else { 8001 };
    let scheme = if is_tls { "wss" } else { "ws" };
    let host_part = host.split(':').next().unwrap_or(host);
    let target_addr = if host.contains(':') { host.to_string() } else { format!("{host_part}:{port}") };

    let url = if let Some(tok) = token {
        format!("{scheme}://{target_addr}/api/v2/channels/{ART_APP_ENDPOINT}?name={APP_NAME_BASE64}&token={tok}")
    } else {
        format!("{scheme}://{target_addr}/api/v2/channels/{ART_APP_ENDPOINT}?name={APP_NAME_BASE64}")
    };

    Ok((url, is_tls, target_addr))
}

fn extract_data(root: &Value) -> Option<Value> {
    let data_val = root.get("data")?;
    if let Some(s) = data_val.as_str() { serde_json::from_str(s).ok() } else { Some(data_val.clone()) }
}

fn generate_id() -> String {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let p = &now as *const _ as usize as u64;
    format!("{:016x}{:016x}", now, p ^ 0x5a5a_5a5a_5a5a_5a5a)
}

fn now_image_date() -> String {
    let secs =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}:{m:02}:{d:02} {:02}:{:02}:{:02}", rem / 3600, rem % 3600 / 60, rem % 60)
}
