//! SSDP local network discovery for Samsung Smart TVs.

use std::collections::BTreeMap;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use crate::error::Result;

const SSDP_MULTICAST: &str = "239.255.255.250:1900";

const M_SEARCH_SAMSUNG: &str = "M-SEARCH * HTTP/1.1\r\n\
HOST: 239.255.255.250:1900\r\n\
MAN: \"ssdp:discover\"\r\n\
MX: 3\r\n\
ST: urn:samsung.com:device:RemoteControlReceiver:1\r\n\
\r\n";

const M_SEARCH_ALL: &str = "M-SEARCH * HTTP/1.1\r\n\
HOST: 239.255.255.250:1900\r\n\
MAN: \"ssdp:discover\"\r\n\
MX: 3\r\n\
ST: ssdp:all\r\n\
\r\n";

#[derive(Clone, Debug, serde::Serialize)]
pub struct DiscoveredTv {
    pub host: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

pub fn discover_tvs(timeout: Duration) -> Result<Vec<DiscoveredTv>> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_millis(300)))?;

    let target: SocketAddr = SSDP_MULTICAST
        .parse()
        .map_err(|e| crate::error::Error::other(format!("Invalid SSDP multicast address: {e}")))?;

    // Broadcast probe messages
    let _ = socket.send_to(M_SEARCH_SAMSUNG.as_bytes(), target);
    let _ = socket.send_to(M_SEARCH_ALL.as_bytes(), target);

    let start = Instant::now();
    let mut results: BTreeMap<String, DiscoveredTv> = BTreeMap::new();
    let mut buf = [0u8; 4096];

    while start.elapsed() < timeout {
        match socket.recv_from(&mut buf) {
            Ok((size, src)) => {
                let response = String::from_utf8_lossy(&buf[..size]);
                if is_samsung_tv(&response) {
                    let host = src.ip().to_string();
                    if !results.contains_key(&host) {
                        let name = extract_header(&response, "SERVER").or_else(|| extract_header(&response, "USN"));
                        let model = extract_model(&response);
                        results.insert(host.clone(), DiscoveredTv { host, name, model });
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                // Read timed out; loop until total deadline
                continue;
            }
            Err(_) => break,
        }
    }

    Ok(results.into_values().collect())
}

fn is_samsung_tv(response: &str) -> bool {
    let lower = response.to_lowercase();
    lower.contains("samsung") || lower.contains("sec_") || lower.contains("remotecontrolreceiver")
}

fn extract_header(response: &str, header: &str) -> Option<String> {
    let prefix = format!("{}:", header.to_lowercase());
    for line in response.lines() {
        let trimmed = line.trim();
        if trimmed.to_lowercase().starts_with(&prefix) {
            let val = trimmed[prefix.len()..].trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

fn extract_model(response: &str) -> Option<String> {
    let loc = extract_header(response, "LOCATION")?;
    // Typical location: http://192.168.1.100:9197/dmr/SamsungMRDesc.xml or /Frame/desc.xml
    // Extract segment before .xml
    if let Some(xml_pos) = loc.find(".xml") {
        let before_xml = &loc[..xml_pos];
        if let Some(slash_pos) = before_xml.rfind('/') {
            let part = &before_xml[slash_pos + 1..];
            if !part.is_empty() {
                return Some(part.to_string());
            }
        }
    }
    None
}
