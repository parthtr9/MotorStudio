//! The Unix-socket transport for the gateway's network API. One request
//! per line, one response per line (see `protocol.rs`). A WebSocket
//! transport is not built yet -- it would reuse `dispatch` unchanged,
//! only the framing (socket accept + line reading) differs.
//!
//! `GatewayServer` wraps a `GatewayDaemon` behind a `std::sync::Mutex`,
//! not `tokio::sync::Mutex`: every daemon call is fully synchronous (no
//! internal `.await`), so the lock is always released before the next
//! `.await` point, satisfying driver/CLAUDE.md's "no locks held across
//! await" -- using the async-aware mutex here would suggest otherwise.

use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::task::JoinHandle;

use crate::backend::Backend;

use super::daemon::GatewayDaemon;
use super::trip_log::{TripLog, read_entries};
use super::types::Enforcer;

pub struct GatewayServer<B: Backend, E: Enforcer> {
    daemon: Mutex<GatewayDaemon<B, E>>,
    trip_log: Mutex<TripLog>,
    trip_log_path: std::path::PathBuf,
}

impl<B: Backend + Send + 'static, E: Enforcer + Send + 'static> GatewayServer<B, E> {
    pub fn new(
        daemon: GatewayDaemon<B, E>,
        trip_log: TripLog,
        trip_log_path: impl AsRef<Path>,
    ) -> Arc<Self> {
        Arc::new(GatewayServer {
            daemon: Mutex::new(daemon),
            trip_log: Mutex::new(trip_log),
            trip_log_path: trip_log_path.as_ref().to_path_buf(),
        })
    }

    /// Binds the Unix socket at `path`, synchronously -- deliberately
    /// split from `serve` below so a caller (notably the test suite)
    /// can know the socket is ready to accept connections the moment
    /// this returns, rather than racing an async task that binds on its
    /// first poll at some scheduler-dependent later time. Removes a
    /// stale socket file left over from a previous run before binding,
    /// so a crashed gateway doesn't block the next start -- that's a
    /// fresh bind, not a claim about whether that previous run was
    /// armed.
    pub fn bind_unix(path: impl AsRef<Path>) -> io::Result<UnixListener> {
        let path = path.as_ref();
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        UnixListener::bind(path)
    }

    /// Accepts connections on an already-bound listener until the
    /// process exits or the listener errors.
    pub async fn serve(self: Arc<Self>, listener: UnixListener) -> io::Result<()> {
        loop {
            let (stream, _addr) = listener.accept().await?;
            let server = Arc::clone(&self);
            tokio::spawn(async move {
                let _ = server.handle_connection(stream).await;
            });
        }
    }

    /// Convenience: bind then serve. Prefer `bind_unix` + `serve`
    /// separately when the caller needs to know bind succeeded before
    /// anything else proceeds (see `bind_unix`'s doc).
    pub async fn serve_unix(self: Arc<Self>, path: impl AsRef<Path>) -> io::Result<()> {
        let listener = Self::bind_unix(path)?;
        self.serve(listener).await
    }

    pub async fn handle_connection(&self, stream: UnixStream) -> io::Result<()> {
        let (read_half, mut write_half) = stream.into_split();
        let mut lines = BufReader::new(read_half).lines();
        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }
            let response = self.dispatch_line(&line);
            let mut out = serde_json::to_string(&response).unwrap_or_else(|_| {
                serde_json::to_string(
                    &json!({"ok": false, "error": "internal serialization error"}),
                )
                .expect("static fallback value always serializes")
            });
            out.push('\n');
            write_half.write_all(out.as_bytes()).await?;
            write_half.flush().await?;
        }
        Ok(())
    }

    fn dispatch_line(&self, line: &str) -> super::protocol::Response {
        use super::protocol::{Request, Response};
        let request: Request = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => return Response::err(format!("invalid request: {e}")),
        };
        self.dispatch(request)
    }

    fn dispatch(&self, request: super::protocol::Request) -> super::protocol::Response {
        use super::protocol::{Request, Response};
        match request {
            Request::Arm => {
                let mut d = self.daemon.lock().unwrap();
                match d.arm(Instant::now()) {
                    Ok(()) => Response::ok(),
                    Err(e) => Response::err(format!("{e:?}")),
                }
            }
            Request::Disarm => {
                let mut d = self.daemon.lock().unwrap();
                match d.disarm() {
                    Ok(()) => Response::ok(),
                    Err(e) => Response::err(format!("{e:?}")),
                }
            }
            Request::Status => {
                let d = self.daemon.lock().unwrap();
                Response::ok_with(json!({"armed": d.is_armed(), "joints": d.joint_names()}))
            }
            Request::Feedback { joint } => {
                let mut d = self.daemon.lock().unwrap();
                match d.feedback(&joint) {
                    Ok(feedback) => Response::ok_with(
                        serde_json::to_value(feedback).unwrap_or(serde_json::Value::Null),
                    ),
                    Err(e) => Response::err(format!("{e:?}")),
                }
            }
            Request::Command { joint, command } => {
                let mut d = self.daemon.lock().unwrap();
                match d.command(&joint, command, Instant::now()) {
                    Ok(trip) => {
                        if let Some(entry) = trip {
                            let _ = self.trip_log.lock().unwrap().append(&entry);
                        }
                        Response::ok()
                    }
                    Err(e) => Response::err(format!("{e:?}")),
                }
            }
            Request::TripLog { limit } => match read_entries(&self.trip_log_path, limit) {
                Ok(entries) => Response::ok_with(
                    serde_json::to_value(entries).unwrap_or(serde_json::Value::Array(vec![])),
                ),
                Err(e) => Response::err(e.to_string()),
            },
        }
    }

    /// Spawns the periodic watchdog task. Runs until the returned handle
    /// is dropped/aborted or the process exits.
    pub fn spawn_watchdog(self: Arc<Self>, tick: Duration) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tick).await;
                let trips = {
                    let mut d = self.daemon.lock().unwrap();
                    d.check_watchdog(Instant::now())
                };
                if !trips.is_empty() {
                    let mut log = self.trip_log.lock().unwrap();
                    for entry in &trips {
                        let _ = log.append(entry);
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{Command, MotorId};
    use crate::backends::mock::MockBackend;
    use crate::gateway::types::{EnforceDecision, FaultPolicy, JointLimits};

    struct PassthroughEnforcer;
    impl Enforcer for PassthroughEnforcer {
        fn enforce(
            &mut self,
            _joint: &str,
            requested: Command,
            _limits: &JointLimits,
        ) -> EnforceDecision {
            EnforceDecision {
                command: requested,
                trip: None,
            }
        }
    }

    fn test_limits() -> JointLimits {
        JointLimits {
            joint: "knee".to_string(),
            min_position_rad: -1.0,
            max_position_rad: 1.0,
            max_velocity_rad_s: 10.0,
            max_torque_nm: 9.0,
            continuous_current_a: 8.0,
            fault_policy: FaultPolicy::ZeroTorque,
            max_winding_temp_degc: None,
        }
    }

    fn test_server(
        tmp_dir: &std::path::Path,
    ) -> Arc<GatewayServer<MockBackend, PassthroughEnforcer>> {
        let backend = MockBackend::new(vec![MotorId(1)]);
        let mut daemon = GatewayDaemon::new(backend, PassthroughEnforcer, Duration::from_secs(60));
        daemon.register_joint("knee", MotorId(1), test_limits());
        let trip_log_path = tmp_dir.join("trip.jsonl");
        let trip_log = TripLog::open(&trip_log_path).unwrap();
        GatewayServer::new(daemon, trip_log, trip_log_path)
    }

    async fn roundtrip<B: Backend + Send + 'static, E: Enforcer + Send + 'static>(
        server: &Arc<GatewayServer<B, E>>,
        request_json: &str,
    ) -> serde_json::Value {
        let (client, server_side) = UnixStream::pair().unwrap();
        let server = Arc::clone(server);
        let handle = tokio::spawn(async move {
            let _ = server.handle_connection(server_side).await;
        });

        let (read_half, mut write_half) = client.into_split();
        write_half
            .write_all(format!("{request_json}\n").as_bytes())
            .await
            .unwrap();
        let mut lines = BufReader::new(read_half).lines();
        let line = lines
            .next_line()
            .await
            .unwrap()
            .expect("expected a response line");
        drop(write_half); // close so handle_connection's read loop ends
        let _ = handle.await;
        serde_json::from_str(&line).unwrap()
    }

    #[tokio::test]
    async fn status_before_arm_reports_not_armed() {
        let dir = tempdir();
        let server = test_server(dir.path());
        let resp = roundtrip(&server, r#"{"cmd":"status"}"#).await;
        assert_eq!(resp["ok"], true);
        assert_eq!(resp["data"]["armed"], false);
    }

    #[tokio::test]
    async fn arm_then_command_round_trip_succeeds() {
        let dir = tempdir();
        let server = test_server(dir.path());
        let arm_resp = roundtrip(&server, r#"{"cmd":"arm"}"#).await;
        assert_eq!(arm_resp["ok"], true);

        let cmd_resp = roundtrip(
            &server,
            r#"{"cmd":"command","joint":"knee","torque_nm":1.0}"#,
        )
        .await;
        assert_eq!(cmd_resp["ok"], true);
    }

    #[tokio::test]
    async fn command_before_arm_is_rejected_over_the_wire() {
        let dir = tempdir();
        let server = test_server(dir.path());
        let resp = roundtrip(
            &server,
            r#"{"cmd":"command","joint":"knee","torque_nm":1.0}"#,
        )
        .await;
        assert_eq!(resp["ok"], false);
    }

    #[tokio::test]
    async fn malformed_json_gets_an_error_response_not_a_dropped_connection() {
        let dir = tempdir();
        let server = test_server(dir.path());
        let resp = roundtrip(&server, "{not valid json").await;
        assert_eq!(resp["ok"], false);
        assert!(resp["error"].as_str().unwrap().contains("invalid request"));
    }

    #[tokio::test]
    async fn trip_log_round_trip_reads_back_written_entries() {
        let dir = tempdir();
        let server = test_server(dir.path());
        roundtrip(&server, r#"{"cmd":"arm"}"#).await;
        {
            // Directly trigger a watchdog trip by setting the daemon's
            // joint clock far in the past, bypassing the wire for setup
            // speed -- what's under test here is the trip_log request
            // reading back what the watchdog already wrote.
            let mut d = server.daemon.lock().unwrap();
            let trips = d.check_watchdog(Instant::now() + Duration::from_secs(120));
            drop(d);
            assert_eq!(trips.len(), 1);
            server.trip_log.lock().unwrap().append(&trips[0]).unwrap();
        }
        let resp = roundtrip(&server, r#"{"cmd":"trip_log"}"#).await;
        assert_eq!(resp["ok"], true);
        assert_eq!(resp["data"].as_array().unwrap().len(), 1);
        assert_eq!(resp["data"][0]["joint"], "knee");
    }

    #[tokio::test]
    async fn end_to_end_over_a_real_bound_unix_socket() {
        let dir = tempdir();
        let socket_path = dir.path().join("gateway.sock");
        let server = test_server(dir.path());
        // Bind synchronously, before spawning the accept loop -- by the
        // time this returns, the socket file exists and the kernel will
        // queue a connection even before `serve`'s first `accept().await`
        // actually runs. No race to retry around.
        let listener =
            GatewayServer::<MockBackend, PassthroughEnforcer>::bind_unix(&socket_path).unwrap();
        let server_handle = tokio::spawn(Arc::clone(&server).serve(listener));

        let client = UnixStream::connect(&socket_path).await.unwrap();
        let (read_half, mut write_half) = client.into_split();
        write_half
            .write_all(b"{\"cmd\":\"status\"}\n")
            .await
            .unwrap();
        let mut lines = BufReader::new(read_half).lines();
        let line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(resp["ok"], true);

        server_handle.abort();
    }

    fn tempdir() -> TempDir {
        TempDir::new()
    }

    struct TempDir {
        path: std::path::PathBuf,
    }

    impl TempDir {
        /// Deliberately short and rooted at `/tmp`, not
        /// `std::env::temp_dir()`: on macOS that resolves to a long
        /// `/var/folders/.../T/` path, which combined with a verbose
        /// directory name exceeds a Unix domain socket's `SUN_LEN`
        /// limit (around 104 bytes) -- a real failure this test suite
        /// hit, not a hypothetical one.
        fn new() -> Self {
            let path = std::path::PathBuf::from("/tmp").join(format!(
                "hgw-{}-{:x}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
                    % 0xFFFFFF
            ));
            std::fs::create_dir_all(&path).unwrap();
            TempDir { path }
        }

        fn path(&self) -> &std::path::Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}
