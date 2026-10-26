# tui-daemon

A terminal UI that monitors and controls a daemon over gRPC. The crate is named
`daemon-controller`; it is built with [ratatui](https://ratatui.rs/) and
[tonic](https://github.com/hyperium/tonic).

## Prerequisites

- A Rust toolchain (edition 2021)
- `protoc` — `build.rs` compiles `proto/daemon.proto` with `tonic-prost-build`

## Build and run

```bash
cargo build --release
cargo run
```

The target daemon address is the first positional argument; it defaults to
`http://127.0.0.1:50051`:

```bash
cargo run -- http://127.0.0.1:50052
```

## Key bindings

| Key | Action |
|---|---|
| `q` / `Q`, `Ctrl-C` | Quit |
| `Tab` / `Shift-Tab` | Move between panes |
| `c` / `C` | Connect to the daemon |
| `d` / `D` | Disconnect |
| `↑` / `k`, `↓` / `j` | Move the selection, or scroll the log pane |
| `Enter` | Run the selected action |

## gRPC contract

`proto/daemon.proto` defines `DaemonService`:

| RPC | Purpose |
|---|---|
| `GetStatus` | Current daemon status |
| `GetMetrics` | Daemon metrics snapshot |
| `Control` | Send a control command |
| `StreamLogs` | Server-streaming log entries |

`build.rs` generates the client only (`build_server(false)`).

## Source layout

```
src/main.rs     # entrypoint, terminal setup, key handling, UI tick loop
src/app.rs      # application state and connection status
src/events.rs   # terminal event handling
src/grpc/       # DaemonClient — generated gRPC client wrapper
src/ui/         # dashboard rendering
proto/          # daemon.proto, compiled by build.rs
```

## Contributing

CI (`.github/workflows/ci.yml`) gates on `cargo fmt --all --check`,
`cargo clippy --all-targets --all-features -- -D warnings`,
`cargo test --all-features`, and `cargo build --release`. Run those locally
before pushing.

Pull requests need a changeset:

```bash
npx changeset add           # with a release note
npx changeset add --empty   # for changes that need no release note
```
