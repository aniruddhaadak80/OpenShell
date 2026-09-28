// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Standalone driver smoke test for the Podman-unavailable diagnostic path.
//!
//! Retry timing and connection failures are covered deterministically by unit
//! tests. This test retains only the executable boundary: argument wiring,
//! process exit status, and the rendered error shown to operators.

use std::process::Stdio;
use std::time::Duration;

#[tokio::test]
async fn missing_podman_socket_exits_with_actionable_diagnostic() {
    let tempdir = tempfile::tempdir().expect("create isolated socket directory");
    let missing_socket = tempdir.path().join("missing-podman.sock");

    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_openshell-driver-podman"));
    command
        .arg("--podman-socket")
        .arg(&missing_socket)
        .kill_on_drop(true)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = tokio::time::timeout(Duration::from_secs(30), command.output())
        .await
        .expect("driver should stop after its bounded retry window")
        .expect("spawn openshell-driver-podman");

    assert!(
        !output.status.success(),
        "driver should exit non-zero when Podman is unreachable"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("connection error"),
        "diagnostic should describe the connection failure:\n{stderr}"
    );
    let compact_stderr = stderr
        .chars()
        .filter(|character| !character.is_whitespace() && *character != '│')
        .collect::<String>();
    let compact_socket = missing_socket
        .display()
        .to_string()
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    assert!(
        compact_stderr.contains(&compact_socket),
        "diagnostic should name the configured socket {}:\n{stderr}",
        missing_socket.display()
    );
}
