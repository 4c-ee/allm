# allm

## allm is currently being developed. It most likely will not work if you clone and build it right now. Please come back later.

**Zero-overhead, terminal-native local-LLM launcher.**

A fast TUI for launching local LLMs. One Rust binary that's a TUI, a daemon, and an OpenAI-compatible proxy. 

This is a stripped, rebranded, and tweaked version of [LlamaStash](https://github.com/llamastash/llamastash) to be substatially smaller, lighter-weight, and slightly differently opinioned.

allm takes a bit more of a Unix philosophy-like approach than LlamaStash; **Do one thing, and do it well.** 

There is no Web UI, no Lemonade, no ds4, no agent skills, no CLI, no model recommender, no HF browser. 

### allm is a service for managing, launching, and serving LLMs, and a TUI for interacting with that service. It does nothing else.

## Contents

- [Why](#why)
- [Install](#install)
- [Configuration](#configuration)
- [Contributing](#contributing)
- [AI Usage, License, and Acknowledgements](#ai-usage)

## Why

Heavy abstractions (Ollama, LM Studio) hide llama.cpp and are resource-intensive.

Raw `llama-server` use is tedious and often confusing. 

LlamaStash has been weighted down by unnessecary features and an overcomplicated codebase.

allm is a fast, transparent launcher that simplifies everything and keeps it clean.


## Install
Get it from the AUR **NOT YET**:
`yay -S allm`

Or build it from source;

`git clone git@github.com:4c-ee/allm.git`

then:

`cargo build -r`

and copy the binary from target/release/allm to your bin (usually `~/.local/bin`), run it with `allm`.


**Note**: This is beta software. Rough edges are to be expected. Windows support is not as well-tested as Linux; Same goes for non-NVIDIA GPUs.
**MacOS support is not tested for allm.**

## Configuration

allm reads `$XDG_CONFIG_HOME/llamastash/config.yaml` on Linux (fallback `~/.config/llamastash/config.yaml`), `~/Library/Application Support/llamastash/config.yaml` on macOS, and `%APPDATA%\llamastash\config\config.yaml` on Windows. 

A fully-annotated example may be found at [`config.example.yaml`](config.example.yaml). The full schema reference is in [`docs/usage.md`](docs/usage.md#configuration).

### Default scan paths

When `model_paths` are empty, allm searches these automatically.

| Service      | Linux                                             | macOS                                                    |
| ----------- | ------------------------------------------------- | -------------------------------------------------------- |
| HuggingFace | `~/.cache/huggingface/hub`                        | `~/Library/Caches/huggingface/hub`                       |
| Ollama      | `~/.ollama/models`                                | `~/.ollama/models`                                       |
| LM Studio   | `~/.lmstudio/models`, `~/.cache/lm-studio/models` | `~/Library/Caches/LMStudio/models`, `~/.lmstudio/models` |

Files anywhere under these roots that end in `.gguf` (and aren't `.gguf.part`) get parsed and added to the catalog.

## Contributing

Bug reports, design discussion, and PRs are very much welcome. Start with [`CONTRIBUTING.md`](CONTRIBUTING.md).

## AI Usage

Multiple AI Coding Harnesses and LLMs were heavily used to create LlamaStash.

allm was developed heavily using [Opencode](https://github.com/anomalyco/opencode) and several LLMs, mostly Minimax m2.7 and m3.

## License

Both LlamaStash and allm are licensed MIT.

## Acknowledgements

LlamaStash is the work of Deepu K Sasidharan.
allm would not be possible without LlamaStash & the work of its contributors.

If you find this tool useful, please consider starring the allm and LlamaStash repos.
