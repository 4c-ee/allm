use anyhow::Result;
use allm::{
  config::loader,
  daemon,
  launch::binary::{locate as locate_binary, LocateInputs},
  tui,
  util::logging,
};

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
  let _ = logging::init(false);
  logging::install_panic_hook();

  let loaded_config = loader::load_config(None);
  let config = &loaded_config.config;
  if let Some(warning) = &loaded_config.warning {
    log::warn!("{warning}");
  }

  let theme = config.theme;
  let custom_palette = None;
  let keymap = tui::keybindings::KeyMap::default();
  let offline = false;
  let mouse_focus = config.mouse_focus;
  let left_pane_ratios = loader::sanitize_left_pane_ratios(&config.left_pane_ratios);

  let mut daemon_opts = daemon::DaemonOptions::from_defaults()?;
  // Resolve `llama-server` so the daemon has its binary when it boots —
  // without this, `start_model` would error out with "no binary
  // resolved" even after a clean daemon start. CLI flag → env →
  // config primary → $PATH, mirroring the priority order the old
  // `daemon start` flow used. A miss is logged and leaves the
  // daemon binary-less; the user surfaces a clearer message at
  // launch time than from a missing-binary panic.
  match locate_binary(LocateInputs {
    cli_flag: None,
    env_var: std::env::var_os("LLAMASTASH_LLAMA_SERVER"),
    config_path: config.backend.llamacpp.primary_binary(),
  }) {
    Ok(p) => daemon_opts.binary = Some(p),
    Err(e) => log::warn!("llama-server lookup failed: {e}"),
  }
  // Carry the config's `backend.llamacpp.servers` extra-build list through
  // to the daemon so its `--list-devices` catalog probes for every
  // configured `llama-server` binary, not just the primary. The server
  // catalog itself is built generically inside `run_foreground` via
  // `Backend::configured_servers` over the live `launch_env.binary`.
  daemon_opts.backend = config.backend.clone();
  daemon_opts.proxy = config.proxy.clone();
  daemon_opts.port_range = config.port_range;
  daemon_opts.probe_timeout_secs = Some(config.probe_timeout_secs);
  daemon_opts.arch_defaults = config.arch_defaults.clone();
  daemon_opts.default_launch_mode = config.default_launch_mode;
  daemon_opts.presets = config.presets.clone();
  daemon_opts.config_path = loader::config_path(None);
  daemon_opts.propagated_cli_args = Vec::new();
  let socket_path = daemon_opts.state_dir.clone();

  // Ensure a daemon is running before the TUI attaches. `start_detached`
  // short-circuits when one is already up (lockfile held + runtime.json
  // present · the historical "connect_or_spawn" flow used by the CLI
  // dispatcher before the CLI was removed), and surfaces a backend
  // precheck refusal as `daemon_start_error` so the TUI can still
  // launch daemon-less with the failure rendered in the Daemon panel.
  let daemon_start_error = match daemon::start_detached(daemon_opts.clone()) {
    Ok(_) => None,
    Err(e) => Some(format!("daemon: {e}")),
  };
  if daemon_start_error.is_none() {
    log::info!("daemon: ready (state dir {})", socket_path.display());
  }

  tui::events::launch(
    theme,
    custom_palette,
    keymap,
    offline,
    mouse_focus,
    left_pane_ratios,
    &socket_path,
    Some(daemon_opts),
    daemon_start_error,
  )
  .await
}
