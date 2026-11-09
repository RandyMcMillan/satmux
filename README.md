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

## Functional parity checks

Run the Rust unit tests with:

```sh
cargo test
```

The tests cover protocol key derivation, encrypted/compressed framing, and
HTTP proxy target parsing. The wire protocol remains compatible with the
Python implementation in `py/satmux.py`.

## License

MIT
