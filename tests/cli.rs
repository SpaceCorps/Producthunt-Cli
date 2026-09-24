//! Integration test suite driving the compiled binary against an in-process mock Apify server.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

#[derive(Clone, Debug)]
struct Recorded {
    method: String,
    path: String,
    #[allow(dead_code)]
    headers: Vec<(String, String)>,
    body: Option<Value>,
}

type Route = (&'static str, &'static str, u16, Value);

struct Mock {
    url: String,
    log: Arc<Mutex<Vec<Recorded>>>,
}

impl Mock {
    fn start(routes: Vec<Route>) -> Mock {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/v2/", listener.local_addr().unwrap());
        let log = Arc::new(Mutex::new(Vec::new()));
        let log2 = log.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let routes = routes.clone();
                let log = log2.clone();
                std::thread::spawn(move || {
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        return;
                    }
                    let mut parts = line.split_whitespace();
                    let method = parts.next().unwrap_or("").to_string();
                    let raw_path = parts.next().unwrap_or("");
                    let path = raw_path.strip_prefix("/v2/").unwrap_or(raw_path).to_string();
                    let mut headers = Vec::new();
                    let mut len = 0usize;
                    loop {
                        let mut h = String::new();
                        if reader.read_line(&mut h).unwrap_or(0) == 0 {
                            break;
                        }
                        let h = h.trim_end();
                        if h.is_empty() {
                            break;
                        }
                        if let Some((k, v)) = h.split_once(':') {
                            let (k, v) = (k.trim().to_lowercase(), v.trim().to_string());
                            if k == "content-length" {
                                len = v.parse().unwrap_or(0);
                            }
                            headers.push((k, v));
                        }
                    }
                    let mut buf = vec![0; len];
                    if len > 0 {
                        let _ = reader.read_exact(&mut buf);
                    }
                    let body = (len > 0).then(|| serde_json::from_slice(&buf).unwrap_or(Value::Null));
                    log.lock().unwrap().push(Recorded { method: method.clone(), path: path.clone(), headers, body });

                    // Match route based on path prefix or equality
                    let matched = routes
                        .iter()
                        .find(|(m, p, _, _)| *m == method && (path == *p || path.starts_with(&format!("{p}?"))));
                    let (status, resp) = matched
                        .map(|(_, _, s, b)| (*s, b.clone()))
                        .unwrap_or((404, json!({"error": {"message": "no route", "type": "NOT_FOUND"}})));

                    let text = if status == 204 { String::new() } else { resp.to_string() };
                    let _ = write!(
                        stream,
                        "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
                        text.len()
                    );
                });
            }
        });
        Mock { url, log }
    }

    fn requests(&self) -> Vec<Recorded> {
        self.log.lock().unwrap().clone()
    }

    fn last(&self, method: &str) -> Recorded {
        self.requests().into_iter().rev().find(|r| r.method == method).expect("no such request")
    }
}

struct Env {
    dir: PathBuf,
    api: String,
}

impl Env {
    fn new(mock: &Mock) -> Env {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "producthunt-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Env { dir, api: mock.url.clone() }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_producthunt"))
            .args(args)
            .env("PRODUCTHUNT_CONFIG_DIR", &self.dir)
            .env("PRODUCTHUNT_SECRET_STORE", "plaintext")
            .env("PRODUCTHUNT_API_URL", &self.api)
            .env_remove("APIFY_TOKEN")
            .output()
            .unwrap()
    }

    fn json(&self, args: &[&str]) -> (i32, Value, Value) {
        let mut all = args.to_vec();
        all.push("--json");
        let out = self.run(&all);
        let parse = |b: &[u8]| {
            let s = String::from_utf8_lossy(b);
            let s = s.lines().filter(|l| !l.starts_with("warning:")).collect::<Vec<_>>().join("\n");
            serde_json::from_str(&s).unwrap_or(Value::Null)
        };
        (out.status.code().unwrap(), parse(&out.stdout), parse(&out.stderr))
    }

    fn with_account(self) -> Env {
        let (code, out, err) = self.json(&["accounts", "add", "work", "--api-key", "apify_token_123"]);
        assert_eq!(code, 0, "{err}");
        assert_eq!(out["status"], "added");
        self
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn agent_readme_markdown_and_json() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);

    let out = env.run(&["agent-readme"]);
    assert_eq!(out.status.code().unwrap(), 0);
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("# producthunt - agent operating manual"));

    let (code, stdout, _) = env.json(&["agent-readme"]);
    assert_eq!(code, 0);
    assert_eq!(stdout["tool"], "producthunt");
    assert_eq!(stdout["actor"], "cloud9_ai~producthunt-scraper");
    assert_eq!(stdout["exitCodes"]["0"], "ok");
    assert_eq!(stdout["exitCodes"]["3"], "auth_required - stop, surface the remediation to a human");
}

#[test]
fn parse_errors_are_envelopes() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);

    let (code, _, err) = env.json(&["search"]);
    assert_eq!(code, 6);
    assert_eq!(err["code"], "invalid_input");
    assert!(err["error"].as_str().unwrap().contains("QUERY"));
}

#[test]
fn account_is_required_when_none_specified() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);

    let (code, _, err) = env.json(&["search", "ai"]);
    assert_eq!(code, 7);
    assert_eq!(err["code"], "no_account");
    assert!(err["remediation"].as_str().unwrap().contains("producthunt login"));
}

#[test]
fn accounts_lifecycle() {
    let mock = Mock::start(vec![("GET", "users/me", 200, json!({"data": {"id": "u_987", "username": "agentuser"}}))]);
    let env = Env::new(&mock);

    let (code, out, err) = env.json(&["accounts", "add", "work", "--api-key", "test_tok"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out["status"], "added");
    assert_eq!(out["name"], "work");
    assert_eq!(out["identity"], "agentuser");
    assert_eq!(out["userId"], "u_987");

    let (code, out, _) = env.json(&["accounts", "list"]);
    assert_eq!(code, 0);
    assert_eq!(out["count"], 1);
    assert_eq!(out["accounts"][0]["name"], "work");
    assert_eq!(out["accounts"][0]["keyStatus"], "stored");

    let (code, out, _) = env.json(&["accounts", "list", "--check"]);
    assert_eq!(code, 0);
    assert_eq!(out["accounts"][0]["keyStatus"], "valid");

    let (code, out, _) = env.json(&["accounts", "test", "work"]);
    assert_eq!(code, 0);
    assert_eq!(out["name"], "work");
    assert_eq!(out["keyStatus"], "valid");

    let (code, out, _) = env.json(&["accounts", "remove", "work", "--yes"]);
    assert_eq!(code, 0);
    assert_eq!(out["status"], "removed");

    let (code, out, _) = env.json(&["accounts", "list"]);
    assert_eq!(code, 0);
    assert_eq!(out["count"], 0);
}

#[test]
fn search_command_executes_scraper() {
    let mock = Mock::start(vec![
        ("GET", "users/me", 200, json!({"data": {"id": "u_1", "username": "agent"}})),
        (
            "POST",
            "acts/cloud9_ai~producthunt-scraper/run-sync-get-dataset-items",
            200,
            json!([
                {
                    "name": "SuperApp",
                    "tagline": "AI pair programmer",
                    "votesCount": 420,
                    "url": "https://www.producthunt.com/posts/superapp"
                }
            ]),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) = env.json(&["search", "super", "--max", "5", "--sort", "newest", "-a", "work"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.is_array());
    assert_eq!(out[0]["name"], "SuperApp");
    assert_eq!(out[0]["votesCount"], 420);

    let recorded = mock.last("POST");
    assert!(recorded.path.contains("acts/cloud9_ai~producthunt-scraper/run-sync-get-dataset-items"));
    let body = recorded.body.unwrap();
    assert_eq!(body["searchQuery"], "super");
    assert_eq!(body["maxResults"], 5);
    assert_eq!(body["sortBy"], "newest");
}

#[test]
fn scrape_command_executes_scraper() {
    let mock = Mock::start(vec![
        ("GET", "users/me", 200, json!({"data": {"id": "u_1", "username": "agent"}})),
        (
            "POST",
            "acts/cloud9_ai~producthunt-scraper/run-sync-get-dataset-items",
            200,
            json!([
                {
                    "name": "TargetPost",
                    "description": "Scraped product details",
                    "url": "https://www.producthunt.com/posts/target"
                }
            ]),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) =
        env.json(&["scrape", "https://www.producthunt.com/posts/target", "--max", "3", "-a", "work"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.is_array());
    assert_eq!(out[0]["name"], "TargetPost");

    let recorded = mock.last("POST");
    let body = recorded.body.unwrap();
    assert_eq!(body["dateUrl"], "https://www.producthunt.com/posts/target");
    assert_eq!(body["maxResults"], 3);
}

#[test]
fn http_errors_map_to_exit_codes() {
    let mock = Mock::start(vec![(
        "GET",
        "users/me",
        401,
        json!({"error": {"message": "Invalid token", "type": "AUTH_ERROR"}}),
    )]);
    let env = Env::new(&mock);

    let (code, _, err) = env.json(&["accounts", "add", "work", "--api-key", "bad_token"]);
    assert_eq!(code, 3);
    assert_eq!(err["code"], "auth_required");
    assert_eq!(err["error"], "Invalid token");
}

#[test]
fn yaml_is_default_output() {
    let mock = Mock::start(vec![
        ("GET", "users/me", 200, json!({"data": {"id": "u_1", "username": "agent"}})),
        (
            "POST",
            "acts/cloud9_ai~producthunt-scraper/run-sync-get-dataset-items",
            200,
            json!([
                {"name": "ToolA", "votes": 100}
            ]),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    let out = env.run(&["search", "tools", "-a", "work"]);
    assert_eq!(out.status.code().unwrap(), 0);
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("name: ToolA"));
    assert!(s.contains("votes: 100"));
}

#[test]
fn apify_token_env_var_works() {
    let mock = Mock::start(vec![(
        "POST",
        "acts/cloud9_ai~producthunt-scraper/run-sync-get-dataset-items",
        200,
        json!([{"name": "EnvResult"}]),
    )]);
    let env = Env::new(&mock);

    let out = Command::new(env!("CARGO_BIN_EXE_producthunt"))
        .args(["search", "ai", "--json"])
        .env("PRODUCTHUNT_CONFIG_DIR", &env.dir)
        .env("PRODUCTHUNT_SECRET_STORE", "plaintext")
        .env("PRODUCTHUNT_API_URL", &env.api)
        .env("APIFY_TOKEN", "env_token_456")
        .output()
        .unwrap();

    assert_eq!(out.status.code().unwrap(), 0);
    let val: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(val[0]["name"], "EnvResult");
}
