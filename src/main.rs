use anyhow::Result;
use llamastash::{config::loader, daemon, theme, tui, util::logging};

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
  let _ = logging::init(false);
  logging::install_panic_hook();

  let _config = loader::load_config(None);

  let theme = theme::ThemeName::Macchiato;
  let custom_palette = None;
  let keymap = tui::keybindings::KeyMap::default();
  let offline = false;
  let mouse_focus = false;
  let left_pane_ratios = vec![65, 100, 50, 35, 0];

  let daemon_opts = daemon::DaemonOptions::from_defaults()?;
  let socket_path = daemon_opts.state_dir.clone();

  tui::events::launch(
    theme,
    custom_palette,
    keymap,
    offline,
    mouse_focus,
    left_pane_ratios,
    &socket_path,
    Some(daemon_opts),
    None,
  )
  .await
}
