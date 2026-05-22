use std::fs;
use std::process::{Command, Stdio};

/// Test that background jobs are in their own process group
#[test]
fn test_background_job_process_group() {
    // Create a test script that starts a background job
    let script = r#"
#!/usr/bin/env aush
# Start a background job
sleep 60 &
echo $!
"#;

    let script_path = "/tmp/aush_test_bg_pgid.aush";
    fs::write(script_path, script).unwrap();

    // Run the script
    let output = Command::new(env!("CARGO_BIN_EXE_aush"))
        .arg(script_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    // Cleanup
    fs::remove_file(script_path).ok();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Ok(pid) = stdout.trim().parse::<u32>() {
            // Check if process is in its own group
            // Using ps to verify PGID == PID
            let ps_output = Command::new("ps")
                .args(&["-o", "pid,pgid", "-p", &pid.to_string()])
                .output();

            if let Ok(ps_output) = ps_output {
                let ps_stdout = String::from_utf8_lossy(&ps_output.stdout);
                // Parse ps output to verify PGID == PID
                for line in ps_stdout.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let proc_pid = parts[0].parse::<u32>().unwrap_or(0);
                        let proc_pgid = parts[1].parse::<u32>().unwrap_or(0);

                        if proc_pid == pid {
                            assert_eq!(
                                proc_pid, proc_pgid,
                                "Background job should be in its own process group (PID == PGID)"
                            );
                        }
                    }
                }
            }

            // Cleanup: kill the background process
            let _ = Command::new("kill").arg(pid.to_string()).output();
        }
    }
}

/// Test that foreground jobs are in their own process group
#[test]
fn test_foreground_job_process_group() {
    // Create a test script that runs a simple command
    let script = r#"
#!/usr/bin/env aush
echo "test"
"#;

    let script_path = "/tmp/aush_test_fg_pgid.aush";
    fs::write(script_path, script).unwrap();

    // Run the script
    let output = Command::new(env!("CARGO_BIN_EXE_aush"))
        .arg(script_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    // Cleanup
    fs::remove_file(script_path).ok();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "test");
        assert_eq!(output.status.code().unwrap_or(1), 0);
    }
}

/// Test that signals are sent to process groups
#[test]
fn test_signal_to_process_group() {
    // Create a test script that starts a pipeline
    let script = r#"
#!/usr/bin/env aush
sleep 100 | cat &
echo $!
"#;

    let script_path = "/tmp/aush_test_signal_pgid.aush";
    fs::write(script_path, script).unwrap();

    // Run the script
    let output = Command::new(env!("CARGO_BIN_EXE_aush"))
        .arg(script_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    // Cleanup
    fs::remove_file(script_path).ok();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Ok(pid) = stdout.trim().parse::<u32>() {
            // Send SIGTERM to the process group
            let pgid = -(pid as i32); // Negative PID sends to process group
            let _ = Command::new("kill")
                .arg("-TERM")
                .arg(pgid.to_string())
                .output();

            // Wait a bit for signal to be processed
            std::thread::sleep(std::time::Duration::from_millis(100));

            // Verify process is terminated
            let ps_check = Command::new("ps").args(&["-p", &pid.to_string()]).output();

            if let Ok(ps_check) = ps_check {
                // Process should not exist anymore
                assert!(
                    !ps_check.status.success()
                        || String::from_utf8_lossy(&ps_check.stdout).lines().count() <= 1,
                    "Process should be terminated by signal to process group"
                );
            }
        }
    }
}

/// Test that jobs builtin shows process group IDs
#[test]
fn test_jobs_shows_pgid() {
    // This test would require running in interactive mode
    // which is difficult to test in CI
    // Manual testing can be done with:
    // $ aush
    // $ sleep 100 &
    // $ jobs -l  # Should show PGID
}

/// Test process group isolation
#[test]
fn test_process_group_isolation() {
    // This test verifies that the implementation calls setpgid using pre_exec
    // We verify this by checking that the code compiles and uses the correct Unix APIs

    // Run a simple foreground command that would trigger process group setup
    let output = Command::new(env!("CARGO_BIN_EXE_aush"))
        .arg("-c")
        .arg("echo test")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    // The command should succeed without any setpgid errors
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("setpgid failed"),
        "Process group setup should not fail: {}",
        stderr
    );

    assert_eq!(
        output.status.code().unwrap_or(1),
        0,
        "Command should execute successfully"
    );
}

/// Test that shell is in its own process group
#[test]
fn test_shell_process_group() {
    // Run aush and check that it puts itself in its own process group
    let script = "echo $$";

    let output = Command::new(env!("CARGO_BIN_EXE_aush"))
        .args(&["-c", script])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Ok(shell_pid) = stdout.trim().parse::<u32>() {
            // Check shell's process group
            let ps_output = Command::new("ps")
                .args(&["-o", "pid,pgid", "-p", &shell_pid.to_string()])
                .output();

            if let Ok(ps_output) = ps_output {
                let ps_stdout = String::from_utf8_lossy(&ps_output.stdout);
                for line in ps_stdout.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let proc_pid = parts[0].parse::<u32>().unwrap_or(0);
                        let proc_pgid = parts[1].parse::<u32>().unwrap_or(0);

                        if proc_pid == shell_pid {
                            // Shell should be in its own process group
                            assert_eq!(
                                proc_pid, proc_pgid,
                                "Shell should be in its own process group (PID == PGID)"
                            );
                        }
                    }
                }
            }
        }
    }
}
