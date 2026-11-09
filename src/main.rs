use anyhow::{Context, Result, bail};
use satmux::{ClientConfig, ServerConfig, run_client, run_server};
use std::env;

const USAGE: &str = "satmux <server|client> [--host HOST] [--port PORT] [--psk-hex HEX] [--carriers N] [--socks-port N] [--http-port N] [--fast-port N] [--gh-sock PATH]";

fn parse_args() -> Result<()> {
    let mut args = env::args().skip(1);
    let mode = args.next().unwrap_or_default();
    if mode == "--help" || mode == "-h" {
        println!("{USAGE}");
        return Ok(());
    }
    if mode != "server" && mode != "client" {
        bail!("usage: {USAGE}");
    }
    let mut host = "0.0.0.0".to_string();
    let mut port = 8443u16;
    let mut carriers = 12usize;
    let mut socks_port = 1080u16;
    let mut http_port = 8080u16;
    let mut fast_port = 8081u16;
    let mut gh_socket =
        env::var("SATMUX_GH_SOCK").unwrap_or_else(|_| "/tmp/satmux-gh-tls.sock".into());
    let mut psk = env::var("SATMUX_PSK")
        .or_else(|_| env::var("ORB_MUX_PSK"))
        .unwrap_or_default();
    while let Some(arg) = args.next() {
        let value = args
            .next()
            .with_context(|| format!("missing value for {arg}"))?;
        match arg.as_str() {
            "--host" => host = value,
            "--port" => port = value.parse()?,
            "--carriers" => carriers = value.parse()?,
            "--socks-port" => socks_port = value.parse()?,
            "--http-port" => http_port = value.parse()?,
            "--fast-port" => fast_port = value.parse()?,
            "--gh-sock" => gh_socket = value,
            "--psk-hex" => psk = value,
            _ => bail!("unknown option: {arg}"),
        }
    }
    if psk.is_empty() {
        bail!("Missing --psk-hex or SATMUX_PSK");
    }
    let psk = decode_hex(&psk)?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    if mode == "server" {
        runtime.block_on(run_server(ServerConfig::new(host, port, psk)))
    } else {
        let mut config = ClientConfig::new(host, port, psk);
        config.carriers = carriers;
        config.socks_port = socks_port;
        config.http_port = http_port;
        config.fast_port = fast_port;
        config.gh_socket = if gh_socket.is_empty() {
            None
        } else {
            Some(gh_socket)
        };
        runtime.block_on(run_client(config))
    }
}

fn decode_hex(s: &str) -> Result<Vec<u8>> {
    if !s.is_ascii() || s.len() % 2 != 0 {
        bail!("PSK hex must contain an even number of ASCII hex digits");
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).context("PSK contains invalid hex"))
        .collect()
}

fn main() {
    if let Err(error) = parse_args() {
        eprintln!("satmux: {error:#}");
        std::process::exit(2);
    }
}
