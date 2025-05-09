use assert_cmd::prelude::*;
use insta::Settings;
use insta_cmd::assert_cmd_snapshot;
use serial_test::serial;
use std::process::Command;

fn standard_filter() -> Settings {
    let mut settings = insta::Settings::clone_current();
    // Remove any color ASCII codes
    settings.add_filter("\x1b", "");
    settings.add_filter("\\[[0-9]+m", "");

    // Filter out the time stamps
    settings.add_filter("[0-9]+:[0-9]+:[0-9]+ ", "12:00:00");
    // The loaded and also total available RAM size can vary
    settings.add_filter("[0-9.]+[MG]B / [0-9.]+[MG]B", "100MB / 300MB");
    // The loading and time can vary
    settings.add_filter("in [0-9]+ ms", "in 10 ms");
    settings
}

#[test]
#[serial]
fn show_corpus_info() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("annis")?;

    cmd.arg("../graphannis/tests/data/")
        .arg("-c")
        .arg("corpus sample-disk-based-3.3")
        .arg("-c")
        .arg("preload")
        .arg("-c")
        .arg("info");

    let settings = standard_filter();
    settings.bind(|| assert_cmd_snapshot!(cmd));

    Ok(())
}

#[test]
#[serial]
fn list_corpora_not_loaded() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("annis")?;

    cmd.arg("../graphannis/tests/data/").arg("-c").arg("list");

    let settings = standard_filter();
    settings.bind(|| assert_cmd_snapshot!(cmd));

    Ok(())
}

#[test]
#[serial]
fn list_corpora_fully_loaded() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("annis")?;

    cmd.arg("../graphannis/tests/data/")
        .arg("-c")
        .arg("corpus sample-disk-based-3.3")
        .arg("-c")
        .arg("preload")
        .arg("-c")
        .arg("list");

    let settings = standard_filter();
    settings.bind(|| assert_cmd_snapshot!(cmd));

    Ok(())
}

#[test]
#[serial]
fn list_corpora_partially_loaded() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("annis")?;

    cmd.arg("../graphannis/tests/data/")
        .arg("-c")
        .arg("corpus sample-disk-based-3.3")
        .arg("-c")
        .arg("count tok")
        .arg("-c")
        .arg("list");

    let settings = standard_filter();
    settings.bind(|| assert_cmd_snapshot!(cmd));

    Ok(())
}
