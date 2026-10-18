//! Runs the built binary with no terminal on stdin, the way scripts and agents do.

use std::process::{Command, Stdio};

fn eeroctl(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_eeroctl"))
        .args(args)
        .env_remove("EERO_SESSION_TOKEN")
        .env_remove("LOG_LEVEL")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

#[test]
fn version_names_the_binary() {
    let out = eeroctl(&["--version"]);
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout.trim(), format!("eeroctl {}", env!("CARGO_PKG_VERSION")));
}

#[test]
fn destructive_commands_refuse_without_yes() {
    for args in [
        &["network", "reboot"][..],
        &["eero", "reboot", "Living Room"],
        &["forward", "delete", "4001"],
        &["network", "set", "--password", "new-wifi-password"],
        &["auth", "logout"],
    ] {
        let mut full = vec!["--user-token", "not-a-real-token"];
        full.extend_from_slice(args);
        let out = eeroctl(&full);
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(!out.status.success(), "{args:?} ran without --yes");
        assert!(
            stderr.contains("without --yes"),
            "{args:?} failed for another reason: {stderr}"
        );
    }
}

#[test]
fn help_does_not_print_the_token() {
    let out = Command::new(env!("CARGO_BIN_EXE_eeroctl"))
        .arg("--help")
        .env("EERO_SESSION_TOKEN", "token-that-must-not-print")
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("EERO_SESSION_TOKEN"));
    assert!(!stdout.contains("token-that-must-not-print"));
}
