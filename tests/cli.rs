use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn top_level_help_lists_public_commands() {
    let mut command = Command::cargo_bin("pc").unwrap();

    command
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("matrix"))
        .stdout(predicate::str::contains("init"));
}

#[test]
fn matrix_help_lists_team_options() {
    let mut command = Command::cargo_bin("pc").unwrap();

    command
        .args(["matrix", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--team"))
        .stdout(predicate::str::contains("--opponents"));
}

#[test]
fn matrix_without_initialized_files_explains_first_run() {
    let config_dir = tempfile::tempdir().unwrap();
    let mut command = Command::cargo_bin("pc").unwrap();

    command
        .env("PC_CONFIG_DIR", config_dir.path())
        .arg("matrix")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Run `pc"))
        .stderr(predicate::str::contains("init` to create sample files"));
}

#[test]
fn init_creates_sample_files() {
    let config_dir = tempfile::tempdir().unwrap();
    let mut command = Command::cargo_bin("pc").unwrap();

    command
        .env("PC_CONFIG_DIR", config_dir.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created sample files"));

    assert!(config_dir.path().join("my-team.txt").exists());
    assert!(config_dir.path().join("opponents.txt").exists());
}

#[test]
fn matrix_rejects_unsupported_showdown_fields_before_opening_tui() {
    let config_dir = tempfile::tempdir().unwrap();
    let team_path = config_dir.path().join("team.txt");
    let opponents_path = config_dir.path().join("opponents.txt");
    std::fs::write(&team_path, "Milotic\nIVs: 0 Atk\n- Recover\n").unwrap();
    std::fs::write(&opponents_path, "Venusaur\n- Sludge Bomb\n").unwrap();

    let mut command = Command::cargo_bin("pc").unwrap();
    command
        .args([
            "matrix",
            "--team",
            team_path.to_str().unwrap(),
            "--opponents",
            opponents_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("IVs are not valid"));
}
