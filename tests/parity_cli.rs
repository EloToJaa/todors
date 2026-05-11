use std::fs;
use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

fn write_fixture() -> (tempfile::TempDir, std::path::PathBuf) {
    let temp = tempdir().expect("temp dir");
    let calendars = temp.path().join("calendars");
    let home = calendars.join("home");
    let work = calendars.join("work");
    fs::create_dir_all(&home).expect("home dir");
    fs::create_dir_all(&work).expect("work dir");

    let vevent = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:event-1\r\nSUMMARY:Team Sync\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
    let vtodo = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VTODO\r\nUID:todo-1\r\nSUMMARY:Buy milk\r\nSTATUS:NEEDS-ACTION\r\nEND:VTODO\r\nEND:VCALENDAR\r\n";

    fs::write(home.join("event.ics"), vevent).expect("write event");
    fs::write(home.join("todo.ics"), vtodo).expect("write todo");

    let config = temp.path().join("config.toml");
    fs::write(
        &config,
        format!(
            "path = \"{}/*\"\ncache_path = \"{}/cache.sqlite3\"\ndefault_command = \"list\"\n",
            calendars.display(),
            temp.path().display()
        ),
    )
    .expect("write config");

    (temp, config)
}

fn has_todoman() -> bool {
    let result = Command::new("todoman").arg("--help").output();
    let Ok(output) = result else {
        return false;
    };
    output.status.success()
}

fn run_cmd(bin: &str, args: &[&str]) -> std::process::Output {
    Command::new(bin).args(args).output().expect("run command")
}

fn parse_first_status_and_summary(json: &str) -> (Option<String>, Option<String>) {
    let value: Value = serde_json::from_str(json).expect("valid json");
    let Some(first) = value.as_array().and_then(|items| items.first()) else {
        return (None, None);
    };
    let status = first
        .get("status")
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let summary = first
        .get("summary")
        .and_then(|value| value.as_str())
        .map(str::to_string);
    (status, summary)
}

fn write_todoman_config(temp: &tempfile::TempDir) -> std::path::PathBuf {
    let config = temp.path().join("todoman_config.py");
    let calendars = temp.path().join("calendars");
    fs::write(
        &config,
        format!(
            "path = \"{}/*\"\ncache_path = \"{}/todoman-cache.sqlite3\"\ndefault_command = \"list\"\n",
            calendars.display(),
            temp.path().display()
        ),
    )
    .expect("write todoman config");
    config
}

#[test]
fn list_only_shows_vtodo_entries() {
    let (_temp, config) = write_fixture();
    let bin = env!("CARGO_BIN_EXE_todors");

    let output = Command::new(bin)
        .arg("--config")
        .arg(&config)
        .arg("list")
        .output()
        .expect("run list");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Buy milk"));
    assert!(!stdout.contains("Team Sync"));
}

#[test]
fn todors_matches_todoman_for_basic_list_if_available() {
    let (temp, config) = write_fixture();
    let bin = env!("CARGO_BIN_EXE_todors");
    let todoman_config = write_todoman_config(&temp);

    if !has_todoman() {
        return;
    }

    let ours = Command::new(bin)
        .arg("--config")
        .arg(&config)
        .arg("list")
        .output()
        .expect("run todors list");
    assert!(ours.status.success());
    let ours_out = String::from_utf8_lossy(&ours.stdout);

    let theirs = Command::new("todoman")
        .arg("--config")
        .arg(&todoman_config)
        .arg("list")
        .output()
        .expect("run todoman list");
    assert!(theirs.status.success());
    let theirs_out = String::from_utf8_lossy(&theirs.stdout);

    assert_eq!(ours_out.lines().count(), theirs_out.lines().count());
}

#[test]
fn porcelain_list_parity_if_todoman_available() {
    let (temp, config) = write_fixture();
    if !has_todoman() {
        return;
    }

    let bin = env!("CARGO_BIN_EXE_todors");
    let todoman_config = write_todoman_config(&temp);

    let ours = run_cmd(
        bin,
        &[
            "--config",
            config.to_str().expect("path"),
            "--porcelain",
            "list",
        ],
    );
    assert!(ours.status.success());
    let ours_text = String::from_utf8_lossy(&ours.stdout);

    let theirs = run_cmd(
        "todoman",
        &[
            "--config",
            todoman_config.to_str().expect("path"),
            "--porcelain",
            "list",
        ],
    );
    assert!(theirs.status.success());
    let theirs_text = String::from_utf8_lossy(&theirs.stdout);

    let ours_pair = parse_first_status_and_summary(&ours_text);
    let theirs_pair = parse_first_status_and_summary(&theirs_text);
    assert_eq!(ours_pair, theirs_pair);
}

#[test]
fn done_and_undo_parity_if_todoman_available() {
    let (temp, config) = write_fixture();
    if !has_todoman() {
        return;
    }

    let bin = env!("CARGO_BIN_EXE_todors");
    let todoman_config = write_todoman_config(&temp);

    let done_ours = run_cmd(
        bin,
        &["--config", config.to_str().expect("path"), "done", "1"],
    );
    assert!(done_ours.status.success());
    let done_theirs = run_cmd(
        "todoman",
        &[
            "--config",
            todoman_config.to_str().expect("path"),
            "done",
            "1",
        ],
    );
    assert!(done_theirs.status.success());

    let ours_list = run_cmd(
        bin,
        &[
            "--config",
            config.to_str().expect("path"),
            "--porcelain",
            "list",
            "--status",
            "ANY",
        ],
    );
    let theirs_list = run_cmd(
        "todoman",
        &[
            "--config",
            todoman_config.to_str().expect("path"),
            "--porcelain",
            "list",
            "--status",
            "ANY",
        ],
    );
    let ours_pair = parse_first_status_and_summary(&String::from_utf8_lossy(&ours_list.stdout));
    let theirs_pair = parse_first_status_and_summary(&String::from_utf8_lossy(&theirs_list.stdout));
    assert_eq!(ours_pair.0, theirs_pair.0);

    let undo_ours = run_cmd(
        bin,
        &["--config", config.to_str().expect("path"), "undo", "1"],
    );
    assert!(undo_ours.status.success());
    let undo_theirs = run_cmd(
        "todoman",
        &[
            "--config",
            todoman_config.to_str().expect("path"),
            "undo",
            "1",
        ],
    );
    assert!(undo_theirs.status.success());
}
