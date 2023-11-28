use assert_cmd::prelude::*;
use insta_cmd::assert_cmd_snapshot;
use std::process::Command;

#[test]
fn show_corpus_info() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("annis")?;

    cmd.arg("../graphannis/tests/data/")
        .arg("-c")
        .arg("corpus sample-disk-based")
        .arg("-c")
        .arg("preload")
        .arg("-c")
        .arg("info");

    let mut settings = insta::Settings::clone_current();
    // Filter out the time stamps
    settings.add_filter("[0-9]+:[0-9]+:[0-9]+ ", "12:00:00");
    // The loaded and also total available RAM size can vary
    settings.add_filter("[0-9.]+ [MG]B / [0-9.]+ [MG]B", "100 / 300 MB");
    // The loading time can vary
    settings.add_filter("Preloaded corpus in [0-9]+ ms", "Preloaded corpus in 9 ms");
    settings.bind(|| assert_cmd_snapshot!(cmd));

    Ok(())
}
