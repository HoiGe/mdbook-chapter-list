use std::process::Command;

#[test]
fn supports_command_exits_successfully_for_supported_renderer() {
    let output = Command::new(env!("CARGO_BIN_EXE_mdbook-chapter-list"))
        .args(["supports", "html"])
        .output()
        .expect("failed to run mdbook-chapter-list");

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn supports_command_exits_unsuccessfully_for_unsupported_renderer() {
    let output = Command::new(env!("CARGO_BIN_EXE_mdbook-chapter-list"))
        .args(["supports", "not-supported"])
        .output()
        .expect("failed to run mdbook-chapter-list");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
