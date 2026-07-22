//! Daemon-level integration tests for the OpenAI-compat proxy
//! listener landed in Unit 1
//! (docs/plans/2026-05-21-001-feat-proxy-router-plan.md).
//!
//! Inline unit tests under `src/proxy/server.rs` cover the
//! `/health` / 501 / keep-alive / port-in-use surface in isolation.
//! These integration tests exercise the same scenarios with the
//! full `run_foreground` daemon up so config wiring + the spawn
//! ordering in `src/daemon/mod.rs` are exercised end-to-end.

#![cfg(feature = "test-fixtures")]

use std::{
  net::{Ipv4Addr, SocketAddr, TcpListener as StdTcpListener},
  path::PathBuf,
  time::Duration,
};

use llamastash::config::loader::ProxyConfig;
use llamastash::daemon::{run_foreground, DaemonOptions};
use llamastash::ipc::Client;
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout};

fn unique_temp_dir(label: &str) -> PathBuf {
  llamastash::test_support::unique_temp_dir("ls-px", label)
}

/// Pick a free loopback port by binding-and-dropping. There's still
/// a TOCTOU window between drop and the daemon's bind, but the
/// tests run on ephemeral kernel-assigned ports so contention is
/// vanishingly unlikely.
fn pick_free_port() -> u16 {
  let l = StdTcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("bind ephemeral");
  l.local_addr().expect("local_addr").port()
}

async fn wait_for_socket(path: &std::path::Path) {
  let deadline = std::time::Instant::now() + Duration::from_secs(10);
  loop {
    if std::time::Instant::now() > deadline {
      panic!(
        "daemon did not become connectable within 10s: {}",
        path.display()
      );
    }
    if Client::connect(path).await.is_ok() {
      return;
    }
    sleep(Duration::from_millis(20)).await;
  }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn daemon_starts_without_proxy_when_disabled() {
  let dir = unique_temp_dir("disabled");
  let mut opts = DaemonOptions::rooted_at(dir.clone());
  let port = pick_free_port();
  opts.proxy = ProxyConfig {
    enabled: false,
    port: Some(port),
    ..ProxyConfig::default()
  };
  let socket_path = opts.state_dir.clone();
  let proxy_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port);
  let handle = tokio::spawn(async move { run_foreground(opts).await });

  wait_for_socket(&socket_path).await;

  // Wait a short beat to let any (incorrect) listener bind, then
  // confirm nothing is answering on the configured port.
  sleep(Duration::from_millis(200)).await;
  let connect_attempt = TcpStream::connect(proxy_addr).await;
  assert!(
    connect_attempt.is_err(),
    "proxy must not be listening when proxy.enabled = false; got {connect_attempt:?}"
  );

  let mut client = Client::connect(&socket_path).await.expect("connect daemon");
  let _ = client.call("shutdown", None).await.expect("shutdown");
  let _ = timeout(Duration::from_secs(3), handle).await;
  std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn daemon_keeps_running_when_proxy_port_already_in_use() {
  let dir = unique_temp_dir("port-in-use");
  // Camp on a port first using std (synchronous) so the daemon
  // observes a guaranteed EADDRINUSE.
  let camp = StdTcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("camp bind");
  let camp_addr: SocketAddr = camp.local_addr().expect("local_addr");
  let port = camp_addr.port();

  let mut opts = DaemonOptions::rooted_at(dir.clone());
  opts.proxy = ProxyConfig {
    enabled: true,
    port: Some(port),
    ..ProxyConfig::default()
  };
  let socket_path = opts.state_dir.clone();
  let handle = tokio::spawn(async move { run_foreground(opts).await });

  // Daemon must reach a connectable IPC socket even though the
  // proxy listener bind failed.
  wait_for_socket(&socket_path).await;

  // A second-level smoke: the IPC `ping` works.
  let mut client = Client::connect(&socket_path).await.expect("connect");
  let pong = client.call("ping", None).await.expect("ping");
  assert_eq!(pong, serde_json::json!("pong"));

  let _ = client.call("shutdown", None).await.expect("shutdown");
  let _ = timeout(Duration::from_secs(3), handle).await;
  drop(camp);
  std::fs::remove_dir_all(&dir).ok();
}
