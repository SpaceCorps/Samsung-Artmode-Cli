//! Offline integration test suite for samsung-artmode CLI with an in-process mock Samsung TV.

use std::io::Read;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};
use tungstenite::Message;

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(1);

struct MockTv {
    url: String,
    #[allow(dead_code)]
    port: u16,
    #[allow(dead_code)]
    received_requests: Arc<Mutex<Vec<Value>>>,
}

impl MockTv {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("ws://127.0.0.1:{port}");
        let received_requests = Arc::new(Mutex::new(Vec::new()));
        let reqs_clone = received_requests.clone();

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };
                let reqs = reqs_clone.clone();

                std::thread::spawn(move || {
                    let mut ws = match tungstenite::accept(stream) {
                        Ok(w) => w,
                        Err(_) => return,
                    };

                    // Send connect event with pairing token
                    let connect_event = json!({
                        "event": "ms.channel.connect",
                        "data": {
                            "token": "mock-tv-token-12345"
                        }
                    });
                    let _ = ws.send(Message::Text(connect_event.to_string()));

                    while let Ok(msg) = ws.read() {
                        match msg {
                            Message::Text(text) => {
                                if let Ok(doc) = serde_json::from_str::<Value>(&text) {
                                    let data_str =
                                        doc.get("params").and_then(|p| p.get("data")).and_then(Value::as_str);

                                    if let Some(ds) = data_str
                                        && let Ok(data) = serde_json::from_str::<Value>(ds)
                                    {
                                        reqs.lock().unwrap().push(data.clone());
                                        let req_name = data.get("request").and_then(Value::as_str).unwrap_or("");

                                        match req_name {
                                            "get_artmode_status" => {
                                                let resp = json!({
                                                    "data": {
                                                        "request": "get_artmode_status",
                                                        "status": "on"
                                                    }
                                                });
                                                let _ = ws.send(Message::Text(resp.to_string()));
                                            }
                                            "get_content_list" => {
                                                let resp = json!({
                                                    "data": {
                                                        "request": "get_content_list",
                                                        "items": [
                                                            { "content_id": "SAM-F001", "name": "Starry Night" },
                                                            { "content_id": "SAM-F002", "name": "Water Lilies" }
                                                        ]
                                                    }
                                                });
                                                let _ = ws.send(Message::Text(resp.to_string()));
                                            }
                                            "get_current_artwork" => {
                                                let resp = json!({
                                                    "data": {
                                                        "request": "get_current_artwork",
                                                        "content_id": "SAM-F001",
                                                        "matte_id": "none"
                                                    }
                                                });
                                                let _ = ws.send(Message::Text(resp.to_string()));
                                            }
                                            "select_image" => {
                                                let resp = json!({
                                                    "data": {
                                                        "request": "select_image",
                                                        "result": "ok"
                                                    }
                                                });
                                                let _ = ws.send(Message::Text(resp.to_string()));
                                            }
                                            "delete_image_list" => {
                                                let resp = json!({
                                                    "data": {
                                                        "request": "delete_image_list",
                                                        "result": "ok"
                                                    }
                                                });
                                                let _ = ws.send(Message::Text(resp.to_string()));
                                            }
                                            "change_favorite" => {
                                                let resp = json!({
                                                    "data": {
                                                        "request": "change_favorite",
                                                        "result": "ok"
                                                    }
                                                });
                                                let _ = ws.send(Message::Text(resp.to_string()));
                                            }
                                            "get_brightness" => {
                                                let resp = json!({
                                                    "data": {
                                                        "request": "get_brightness",
                                                        "value": 7
                                                    }
                                                });
                                                let _ = ws.send(Message::Text(resp.to_string()));
                                            }
                                            "set_brightness" => {
                                                let resp = json!({
                                                    "data": {
                                                        "request": "set_brightness",
                                                        "result": "ok"
                                                    }
                                                });
                                                let _ = ws.send(Message::Text(resp.to_string()));
                                            }
                                            "get_matte_list" => {
                                                let resp = json!({
                                                    "data": {
                                                        "request": "get_matte_list",
                                                        "mattes": ["none", "flexible_polar", "shadowbox_black"]
                                                    }
                                                });
                                                let _ = ws.send(Message::Text(resp.to_string()));
                                            }
                                            "change_matte" => {
                                                let resp = json!({
                                                    "data": {
                                                        "request": "change_matte",
                                                        "result": "ok"
                                                    }
                                                });
                                                let _ = ws.send(Message::Text(resp.to_string()));
                                            }
                                            "set_slideshow_status" => {
                                                let resp = json!({
                                                    "data": {
                                                        "request": "set_slideshow_status",
                                                        "result": "ok"
                                                    }
                                                });
                                                let _ = ws.send(Message::Text(resp.to_string()));
                                            }
                                            "send_image" => {
                                                // Spawn mock TCP binary receiver
                                                let tcp_listener = TcpListener::bind("127.0.0.1:0").unwrap();
                                                let tcp_port = tcp_listener.local_addr().unwrap().port();

                                                let ready_msg = json!({
                                                    "data": {
                                                        "event": "ready_to_use",
                                                        "conn_info": {
                                                            "ip": "127.0.0.1",
                                                            "port": tcp_port,
                                                            "secured": false,
                                                            "key": "mock-sec-key"
                                                        }
                                                    }
                                                });
                                                let _ = ws.send(Message::Text(ready_msg.to_string()));

                                                // Accept binary payload
                                                if let Ok((mut tcp_stream, _)) = tcp_listener.accept() {
                                                    let mut len_bytes = [0u8; 4];
                                                    if tcp_stream.read_exact(&mut len_bytes).is_ok() {
                                                        let header_len = u32::from_be_bytes(len_bytes) as usize;
                                                        let mut header_buf = vec![0u8; header_len];
                                                        let _ = tcp_stream.read_exact(&mut header_buf);
                                                        let mut body_buf = Vec::new();
                                                        let _ = tcp_stream.read_to_end(&mut body_buf);
                                                    }
                                                }

                                                // Send image_added confirmation
                                                let added_msg = json!({
                                                    "data": {
                                                        "event": "image_added",
                                                        "content_id": "SAM-UPLOADDONE"
                                                    }
                                                });
                                                let _ = ws.send(Message::Text(added_msg.to_string()));
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            Message::Close(_) => break,
                            _ => {}
                        }
                    }
                });
            }
        });

        // Give listener a moment to start
        std::thread::sleep(Duration::from_millis(20));

        MockTv { url, port, received_requests }
    }
}

struct TestEnv {
    dir: PathBuf,
}

impl TestEnv {
    fn new() -> Self {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("samsung-artmode-test-{}-{}", std::process::id(), n));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        TestEnv { dir }
    }

    fn run(&self, args: &[&str]) -> Output {
        let bin = env!("CARGO_BIN_EXE_samsung-artmode");
        Command::new(bin)
            .args(args)
            .env("SAMSUNG_CONFIG_DIR", &self.dir)
            .env("SAMSUNG_SECRET_STORE", "plaintext")
            .env("SAMSUNG_ALLOW_PLAINTEXT_STORE", "1")
            .env_remove("SAMSUNG_TV_HOST")
            .env_remove("SAMSUNG_TV_TOKEN")
            .output()
            .unwrap()
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn stdout_json(out: &Output) -> Value {
    let s = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str(&s).unwrap_or_else(|e| panic!("stdout not json: {e}\nstdout: {s}"))
}

fn stderr_json(out: &Output) -> Value {
    let s = String::from_utf8_lossy(&out.stderr);
    serde_json::from_str(&s).unwrap_or_else(|e| panic!("stderr not json: {e}\nstderr: {s}"))
}

// ---------------------------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------------------------

#[test]
fn agent_readme_markdown_and_json() {
    let env = TestEnv::new();

    // Default: markdown
    let out = env.run(&["agent-readme"]);
    assert!(out.status.success());
    let md = String::from_utf8_lossy(&out.stdout);
    assert!(md.contains("# samsung-artmode - agent operating manual"));
    assert!(md.contains("Target Device Resolution"));

    // With --json
    let out = env.run(&["agent-readme", "--json"]);
    assert!(out.status.success());
    let j = stdout_json(&out);
    assert_eq!(j["tool"], "samsung-artmode");
    assert_eq!(j["apiVersion"], "1.0.0");
    assert!(j["rules"].is_array());
    assert_eq!(j["exitCodes"]["0"], "ok");
    assert_eq!(j["exitCodes"]["3"], "auth_required - stop, surface the remediation to a human");
}

#[test]
fn accounts_lifecycle() {
    let env = TestEnv::new();

    // 1. Initial list is empty
    let out = env.run(&["accounts", "list", "--json"]);
    assert!(out.status.success());
    let j = stdout_json(&out);
    assert_eq!(j["accounts"].as_array().unwrap().len(), 0);

    // 2. Add an account
    let out = env.run(&[
        "accounts",
        "add",
        "living-room",
        "--host",
        "192.168.1.50",
        "--token",
        "secret-token",
        "--model",
        "TheFrame-2024",
        "--json",
    ]);
    assert!(out.status.success());
    let j = stdout_json(&out);
    assert_eq!(j["status"], "ok");
    assert_eq!(j["account"], "living-room");

    // 3. Adding again without --force fails
    let out = env.run(&["accounts", "add", "living-room", "--host", "192.168.1.50", "--json"]);
    assert_eq!(out.status.code(), Some(6)); // invalid_input
    let err = stderr_json(&out);
    assert_eq!(err["code"], "invalid_input");

    // 4. List now has 1 account
    let out = env.run(&["accounts", "list", "--json"]);
    assert!(out.status.success());
    let j = stdout_json(&out);
    assert_eq!(j["accounts"].as_array().unwrap().len(), 1);
    assert_eq!(j["accounts"][0]["name"], "living-room");
    assert_eq!(j["accounts"][0]["host"], "192.168.1.50");

    // 5. Remove account
    let out = env.run(&["accounts", "remove", "living-room", "--yes", "--json"]);
    assert!(out.status.success());
    let j = stdout_json(&out);
    assert_eq!(j["status"], "ok");
    assert_eq!(j["removed"], "living-room");
}

#[test]
fn pair_command_with_mock_tv() {
    let tv = MockTv::start();
    let env = TestEnv::new();

    let out = env.run(&["pair", &tv.url, "--name", "living-room", "--json"]);
    assert!(out.status.success());
    let j = stdout_json(&out);
    assert_eq!(j["status"], "ok");
    assert_eq!(j["token"], "mock-tv-token-12345");
    assert_eq!(j["account"], "living-room");

    // Check account was added
    let out = env.run(&["accounts", "list", "--json"]);
    let j = stdout_json(&out);
    assert_eq!(j["accounts"].as_array().unwrap().len(), 1);
}

#[test]
fn artmode_status_and_list() {
    let tv = MockTv::start();
    let env = TestEnv::new();

    // Query status with --host
    let out = env.run(&["status", "--host", &tv.url, "--json"]);
    assert!(out.status.success());
    let j = stdout_json(&out);
    assert_eq!(j["status"], "on");

    // Query list
    let out = env.run(&["list", "--host", &tv.url, "--category", "MY-C0002", "--json"]);
    assert!(out.status.success());
    let j = stdout_json(&out);
    assert!(j["items"].is_array());
    assert_eq!(j["items"].as_array().unwrap().len(), 2);
}

#[test]
fn current_and_select() {
    let tv = MockTv::start();
    let env = TestEnv::new();

    // current
    let out = env.run(&["current", "--host", &tv.url, "--json"]);
    assert!(out.status.success());
    let j = stdout_json(&out);
    assert_eq!(j["content_id"], "SAM-F001");

    // select
    let out = env.run(&["select", "SAM-F002", "--host", &tv.url, "--json"]);
    assert!(out.status.success());
    let j = stdout_json(&out);
    assert_eq!(j["result"], "ok");
}

#[test]
fn brightness_commands() {
    let tv = MockTv::start();
    let env = TestEnv::new();

    // brightness get
    let out = env.run(&["brightness", "get", "--host", &tv.url, "--json"]);
    assert!(out.status.success());
    let j = stdout_json(&out);
    assert_eq!(j["value"], 7);

    // brightness set
    let out = env.run(&["brightness", "set", "9", "--host", &tv.url, "--json"]);
    assert!(out.status.success());
    let j = stdout_json(&out);
    assert_eq!(j["result"], "ok");

    // brightness set out-of-range fails
    let out = env.run(&["brightness", "set", "15", "--host", &tv.url, "--json"]);
    assert_eq!(out.status.code(), Some(6));
    let err = stderr_json(&out);
    assert_eq!(err["code"], "invalid_input");
}

#[test]
fn matte_and_favorite_commands() {
    let tv = MockTv::start();
    let env = TestEnv::new();

    // matte list
    let out = env.run(&["matte", "list", "--host", &tv.url, "--json"]);
    assert!(out.status.success());
    let j = stdout_json(&out);
    assert!(j["mattes"].is_array());

    // matte set
    let out = env.run(&["matte", "set", "SAM-F001", "flexible_polar", "--host", &tv.url, "--json"]);
    assert!(out.status.success());
    let j = stdout_json(&out);
    assert_eq!(j["result"], "ok");

    // favorite
    let out = env.run(&["favorite", "SAM-F001", "--host", &tv.url, "--json"]);
    assert!(out.status.success());
    let j = stdout_json(&out);
    assert_eq!(j["result"], "ok");
}

#[test]
fn slideshow_and_delete() {
    let tv = MockTv::start();
    let env = TestEnv::new();

    // slideshow configure
    let out = env.run(&["slideshow", "--interval", "30", "--shuffle", "--host", &tv.url, "--json"]);
    assert!(out.status.success());
    let j = stdout_json(&out);
    assert_eq!(j["result"], "ok");

    // slideshow off
    let out = env.run(&["slideshow", "--off", "--host", &tv.url, "--json"]);
    assert!(out.status.success());
    let j = stdout_json(&out);
    assert_eq!(j["result"], "ok");

    // delete
    let out = env.run(&["delete", "SAM-F001", "--host", &tv.url, "--json"]);
    assert!(out.status.success());
    let j = stdout_json(&out);
    assert_eq!(j["status"], "ok");
    assert_eq!(j["deleted"], "SAM-F001");
}

#[test]
fn upload_image_flow() {
    let tv = MockTv::start();
    let env = TestEnv::new();

    // Create a temporary mock image file
    let img_path = env.dir.join("test-artwork.jpg");
    std::fs::write(&img_path, vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46]).unwrap();

    let out = env.run(&[
        "upload",
        img_path.to_str().unwrap(),
        "--matte",
        "shadowbox_black",
        "--select",
        "--host",
        &tv.url,
        "--json",
    ]);
    assert!(out.status.success());
    let j = stdout_json(&out);
    assert_eq!(j["status"], "ok");
    assert_eq!(j["contentId"], "SAM-UPLOADDONE");
    assert_eq!(j["selected"], true);
}

#[test]
fn missing_tv_account_reports_no_account() {
    let env = TestEnv::new();

    let out = env.run(&["status", "--json"]);
    assert_eq!(out.status.code(), Some(7)); // no_account
    let err = stderr_json(&out);
    assert_eq!(err["code"], "no_account");
    assert!(err["detail"].as_str().unwrap().contains("No TV accounts are configured yet"));
}

#[test]
fn test_login_command() {
    let env = TestEnv::new();

    // Verify `samsung-artmode login --help` succeeds
    let out = env.run(&["login", "--help"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Log in / pair with a Samsung Frame TV"));
}

