use assert_cmd::cargo;
use assert_cmd::prelude::*;
use insta::Settings;
use insta::assert_snapshot;
use serial_test::serial;
use std::path::Path;

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

fn mimic_insta_snapshot(
    output: std::process::Output,
) -> Result<String, Box<dyn std::error::Error>> {
    let success = output.status.success();
    let exit_code = output.status.code().unwrap();
    let stdout = str::from_utf8(&output.stdout)?;
    let stderr = str::from_utf8(&output.stderr)?;
    Ok(format!(
        r#"
success: {success}
exit_code: {exit_code}
----- stdout -----
{stdout}
----- stderr -----
{stderr}
    "#
    ))
}

#[test]
#[serial]
fn show_corpus_info() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo::cargo_bin_cmd!("annis");

    cmd.arg("../graphannis/tests/data/")
        .arg("-c")
        .arg("corpus sample-disk-based-3.8")
        .arg("-c")
        .arg("preload")
        .arg("-c")
        .arg("info");

    let output = cmd.output().unwrap();
    let actual = mimic_insta_snapshot(output)?;
    let settings = standard_filter();
    settings.bind(|| assert_snapshot!(actual));

    Ok(())
}

#[test]
#[serial]
fn list_corpora_not_loaded() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo::cargo_bin_cmd!("annis");

    cmd.arg("../graphannis/tests/data/").arg("-c").arg("list");

    let output = cmd.output().unwrap();
    let actual = mimic_insta_snapshot(output)?;
    let settings = standard_filter();
    settings.bind(|| assert_snapshot!(actual));

    Ok(())
}

#[test]
#[serial]
fn list_corpora_fully_loaded() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo::cargo_bin_cmd!("annis");

    cmd.arg("../graphannis/tests/data/")
        .arg("-c")
        .arg("corpus sample-disk-based-3.3")
        .arg("-c")
        .arg("preload")
        .arg("-c")
        .arg("list");

    let output = cmd.output().unwrap();
    let actual = mimic_insta_snapshot(output)?;
    let settings = standard_filter();
    settings.bind(|| assert_snapshot!(actual));

    Ok(())
}

#[test]
#[serial]
fn list_corpora_partially_loaded() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo::cargo_bin_cmd!("annis");

    cmd.arg("../graphannis/tests/data/")
        .arg("-c")
        .arg("corpus sample-disk-based-3.3")
        .arg("-c")
        .arg("count tok")
        .arg("-c")
        .arg("list");

    let output = cmd.output().unwrap();
    let actual = mimic_insta_snapshot(output)?;
    let settings = standard_filter();
    settings.bind(|| assert_snapshot!(actual));

    Ok(())
}

#[test]
#[serial]
fn export_to_zip_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo::cargo_bin_cmd!("annis");

    cmd.arg("../graphannis/tests/data/")
        .arg("-c")
        .arg("corpus sample-disk-based-3.3")
        .arg("-c")
        .arg("export sample-disk-based-3.3.zip");

    let output = cmd.output().unwrap();
    let actual = mimic_insta_snapshot(output)?;
    let settings = standard_filter();
    settings.bind(|| assert_snapshot!(actual));

    // Check that the file has been created
    let p = Path::new("sample-disk-based-3.3.zip");
    assert_eq!(true, p.is_file());
    // Cleanup created file
    std::fs::remove_file(p)?;
    Ok(())
}
