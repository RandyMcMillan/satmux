# satmux

Rust implementation of the TCP multiplexer and HTTP/TLS relay. The original
Python implementation, helpers, and documentation are preserved under [`py/`](py/).

## Build and run

Create a 32-byte pre-shared key as hexadecimal and keep it private:

```sh
openssl rand -hex 32 > mux.psk
chmod 600 mux.psk
```

Run the ground server:

```sh
SATMUX_PSK="$(cat mux.psk)" cargo run --release -- server --host 0.0.0.0 --port 8444
```

Run the local client:

```sh
SATMUX_PSK="$(cat mux.psk)" cargo run --release -- client \
  --host <SERVER_IP> --port 8444 --carriers 12
```

The client listens on SOCKS5 `127.0.0.1:1080`, HTTP proxy
`127.0.0.1:8080`, and the 1-RTT fetch gateway `127.0.0.1:8081`. The optional
GitHub CLI TLS bridge uses `/tmp/satmux-gh-tls.sock` by default; pass
`--gh-sock ""` to disable it. `SATMUX_PSK` or `ORB_MUX_PSK` may supply the key,
or use `--psk-hex`.

The original helper scripts remain available as `py/bin/satmux-bench`,
`py/bin/satmux-get`, and `py/bin/satmux-macos`.

## Use as a library

The package provides both the `satmux` library and the `satmux` binary. Add it
to another crate using a local path (adjust the path to this checkout):

```toml
[dependencies]
anyhow = "1"
satmux = { path = "../satmux" }
tokio = { version = "1", features = ["full"] }
```

Call the async entry points from your application's Tokio runtime:

```rust,no_run
use satmux::{ClientConfig, ServerConfig, run_client, run_server};

async fn relay(psk: Vec<u8>) -> anyhow::Result<()> {
    let server = ServerConfig::new("0.0.0.0", 8444, psk.clone());
    let mut client = ClientConfig::new("127.0.0.1", 8444, psk);
    client.carriers = 4;
    client.socks_port = 1081;

    tokio::try_join!(run_server(server), run_client(client))?;
    Ok(())
}
```

Keys are raw bytes, not hex strings. Library calls do not read environment
variables, create a runtime, or exit the process; errors are returned to the
caller. `ClientConfig::new` uses 12 carriers and the usual TCP proxy ports, but
disables the Unix socket bridge unless `gh_socket` is explicitly set. The CLI
retains its existing defaults, environment variables, and flags.

Both entry points run until an error or cancellation. Drop their futures (or
abort their Tokio tasks) to close TCP listeners and abort their background
connections. An enabled Unix socket may leave its socket file behind after
cancellation; a subsequent client start removes it.

## Functional parity checks

Run the Rust unit tests with:

```sh
cargo test
```

The tests cover protocol key derivation, encrypted/compressed framing, a live
TCP tunnel, public library configuration and cancellation, and CLI compatibility.
The library example is also compiled as a doctest. The wire protocol remains
compatible with the Python implementation in `py/satmux.py`.

## License

MIT
