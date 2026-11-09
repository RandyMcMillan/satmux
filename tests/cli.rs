use std::process::Command;

fn satmux() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_satmux"));
    command
        .env_remove("SATMUX_PSK")
        .env_remove("ORB_MUX_PSK")
        .env_remove("SATMUX_GH_SOCK");
    command
}

#[test]
fn help_is_available_without_a_key() {
    for flag in ["--help", "-h"] {
        let output = satmux().arg(flag).output().unwrap();
        assert!(output.status.success());
        let help = String::from_utf8(output.stdout).unwrap();
        for option in [
            "<server|client>",
            "--host",
            "--port",
            "--psk-hex",
            "--carriers",
            "--socks-port",
            "--http-port",
            "--fast-port",
            "--gh-sock",
        ] {
            assert!(help.contains(option), "missing help for {option}");
        }
    }
}

#[test]
fn missing_key_still_exits_with_an_explicit_error() {
    let output = satmux().arg("server").output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Missing --psk-hex or SATMUX_PSK"));
}

#[test]
fn malformed_keys_are_errors_not_panics() {
    for key in ["f", "zz", "\u{20ac}a"] {
        let output = satmux()
            .args(["server", "--psk-hex", key])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("PSK"), "{error}");
        assert!(!error.contains("panicked"), "{error}");
    }
}

#[test]
fn explicit_key_overrides_the_environment() {
    let output = satmux()
        .env("SATMUX_PSK", "invalid environment key")
        .args(["client", "--psk-hex", "42", "--carriers", "0"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("at least one carrier"));
}
