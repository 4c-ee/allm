# allm

## allm is currently being developed. It most likely will not work if you clone and build it right now. Please come back later.

**Zero-overhead, terminal-native local-LLM launcher.**

A fast TUI for launching local LLMs. One Rust binary that's a TUI, a daemon, and an OpenAI-compatible proxy. 

This is a stripped, rebranded, and tweaked version of [LlamaStash](github.com/llamastash/llamastash) to be substatially smaller, lighter-weight, and slightly differently opinioned.
allm takes a bit more Unix philosophy-like approach. **Do one thing, and do it well.** There is no Web UI, no Lemonade, no ds4, no agent skills, no CLI, no model recommender, no HF browser.
### allm is a service for managing, launching, and serving LLMs, and a TUI for interacting with that service. It does nothing else.

## Contents

- [Why](#why)
- [Install](#install)
- [Configuration](#configuration)
- [Platforms](#platforms)
- [Contributing](#contributing)

## Why

Heavy abstractions (Ollama, LM Studio) hide llama.cpp and are resource-intensive.
Raw `llama-server` use is tedious and sometimes confusing. 
allm is a fast, transparent launcher that simplifies everything and keeps it clean.


## Install
Get it from the AUR **NOT YET**:
`yay -S allm`

Or build it from source:
`git clone git@github.com:4c-ee/allm.git`
then:
`cargo build -r`
and copy the binary from target/release/allm to your bin (usually `~/.local/bin`), run it with `allm`.


**Note**: This is beta software. Rough edges are to be expected. Windows support is not as well-tested as Linux; Same goes for non-NVIDIA GPUs.
**MacOS support is not tested and likely nonfunctional.**

## Configuration

LlamaStash reads `$XDG_CONFIG_HOME/llamastash/config.yaml` on Linux (fallback `~/.config/llamastash/config.yaml`), `~/Library/Application Support/llamastash/config.yaml` on macOS, and `%APPDATA%\llamastash\config\config.yaml` on Windows. A fully-annotated sample lives at [`config.example.yaml`](config.example.yaml) — copy it to the path above and edit. Run `llamastash config` to open the active file in `$EDITOR`, or `llamastash config bindings` to print every effective keybinding as YAML. The full schema reference is in [`docs/usage.md`](docs/usage.md#configuration).

Quick tour of the top-level keys:

| Key                           | What it controls                                                                                                                                                          |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `theme`                       | Built-in palette: `macchiato` (default), `latte`, `gruvbox-dark`, `solarized-dark`, `mono`. Set to `custom` to use the `custom_theme` block. Cycle live with `t:theme`.   |
| `custom_theme`                | User-defined palette. Inherits unspecified slots from `base:` (default macchiato). Accepts `#RRGGBB` hex or ANSI names. Once defined, `Custom` joins the `t:theme` cycle. |
| `model_paths`                 | Extra directories to scan for `.gguf` files. Merged with `-p/--model-path` and `LLAMASTASH_MODEL_PATHS`.                                                                  |
| `disable_default_cache_paths` | Per-bucket toggles (`huggingface`, `ollama`, `lm_studio`) for the auto-walked caches.                                                                                     |
| `disable_scan`                | Skip filesystem scanning entirely. Same as `--no-scan` / `LLAMASTASH_NO_SCAN=1`.                                                                                          |
| `port_range`                  | Inclusive `{start, end}` TCP range the supervisor picks from. Default `41100..=41300`.                                                                                    |
| `backend.llamacpp.servers`     | `llama-server` build/binary variants (`[{binary, name?}]`). First = default; each is a selectable "server". `--llama-server` / `LLAMASTASH_LLAMA_SERVER` set the first.   |
| `probe_timeout_secs`          | Health-probe deadline per launch. Default `120`. Bump for 70B+ on slow disks.                                                                                             |
| `keybindings`                 | Action-name → key-spec overrides. Kdash-style dialect (`ctrl+q`, `shift+tab`, `f1`, …).                                                                                   |

### Default scan paths

When `model_paths` and `--model-path` are empty, allm searches these automatically.

| Service      | Linux                                             | macOS                                                    |
| ----------- | ------------------------------------------------- | -------------------------------------------------------- |
| HuggingFace | `~/.cache/huggingface/hub`                        | `~/Library/Caches/huggingface/hub`                       |
| Ollama      | `~/.ollama/models`                                | `~/.ollama/models`                                       |
| LM Studio   | `~/.lmstudio/models`, `~/.cache/lm-studio/models` | `~/Library/Caches/LMStudio/models`, `~/.lmstudio/models` |

Files anywhere under these roots that end in `.gguf` (and aren't `.gguf.part`) get parsed and added to the catalog.

## Contributing

Bug reports, design discussion, and PRs welcome. Start with [`CONTRIBUTING.md`](CONTRIBUTING.md).

## AI Usage

Multiple AI Coding Harnesses and LLMs were heavily used to create llamastash.
allm was developed using opencode and several LLMs including Minimax m2.7 and m3.

## License

Both LlamaStash and allm are licensed MIT.

## Acknowledgements
LlamaStash is the work of Deepu K Sasidharan.
allm would not be possible without LlamaStash & the work of its contributors.

If you find these tools useful, please consider starring the allm and LlamaStash repos.
