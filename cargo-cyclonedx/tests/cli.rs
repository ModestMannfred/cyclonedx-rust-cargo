use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn find_content_in_help_output() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("cyclonedx").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("cargo cyclonedx [OPTIONS]"));

    Ok(())
}

#[test]
fn manifest_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = assert_fs::TempDir::new()?;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.current_dir(tmp_dir.path()).arg("cyclonedx");

    cmd.assert().failure().stdout("");

    tmp_dir.close()?;

    Ok(())
}

#[test]
fn manifest_is_invalid() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_file = assert_fs::NamedTempFile::new("Cargo.toml")?;
    tmp_file.touch()?;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.arg("cyclonedx")
        .arg("--manifest-path")
        .arg(tmp_file.path());

    cmd.assert().failure().stdout("");

    tmp_file.close()?;

    Ok(())
}

#[test]
fn find_content_in_bom_files() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = make_temp_rust_project()?;
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.current_dir(tmp_dir.path())
        .arg("cyclonedx")
        .arg("--top-level");

    cmd.assert().success().stdout("");

    tmp_dir
        .child("bom.xml")
        .assert(predicate::str::contains("<vendor>CycloneDX</vendor>"));

    cmd.arg("--format").arg("json");
    cmd.assert().success().stdout("");

    tmp_dir
        .child("bom.json")
        .assert(predicate::str::contains(r#""vendor": "CycloneDX"#));

    tmp_dir.close()?;

    Ok(())
}

#[test]
fn find_content_in_stderr() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = make_temp_rust_project()?;

    let pkg_name = "nested-pkg";

    tmp_dir.child("Cargo.toml").write_str(&format!(
        r#"
        [package]
        name = "test"
        version = "0.0.0"

        [dependencies.{0}]
        path = "{0}"
        "#,
        pkg_name,
    ))?;

    let license = "TEST";
    let pkg_dir = tmp_dir.child(pkg_name);
    pkg_dir.child("src/lib.rs").touch()?;

    pkg_dir.child("Cargo.toml").write_str(&format!(
        r#"
        [package]
        name = "{}"
        version = "0.0.0"
        license = "{}"
        "#,
        pkg_name, license,
    ))?;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.current_dir(tmp_dir.path())
        .arg("cyclonedx")
        .arg("--all")
        .arg("--license-strict")
        .arg("--verbose");

    cmd.assert()
        .success()
        .stderr(predicate::str::contains(format!(
            "Outputting {}",
            tmp_dir.path().join("bom.xml").display(),
        )))
        .stderr(predicate::str::contains(format!(
            "Package {} has an invalid license expression ({}), using as named license: Invalid SPDX expression: unknown term",
            pkg_name, license,
        )));

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.current_dir(tmp_dir.path())
        .arg("cyclonedx")
        .arg("--quiet");

    cmd.assert().success().stdout("").stderr("");

    tmp_dir.close()?;

    Ok(())
}

#[test]
fn bom_file_name_extension_is_prepended_with_cdx() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = make_temp_rust_project()?;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.current_dir(tmp_dir.path())
        .arg("cyclonedx")
        .arg("--output-cdx");

    cmd.assert().success().stdout("");

    tmp_dir.child("bom.xml").assert(predicate::path::missing());

    tmp_dir
        .child("bom.cdx.xml")
        .assert(predicate::path::exists());

    tmp_dir.close()?;

    Ok(())
}

fn make_temp_rust_project() -> Result<assert_fs::TempDir, assert_fs::fixture::FixtureError> {
    let tmp_dir = assert_fs::TempDir::new()?;
    tmp_dir.child("src/main.rs").touch()?;

    tmp_dir
        .child("Cargo.toml")
        .write_str(r#"package = { name = "pkg", version = "0.0.0" }"#)?;

    Ok(tmp_dir)
}

#[test]
fn test_extended_metadata() -> Result<(), Box<dyn std::error::Error>> {
    const OUTPUT_PREFIX: &str = "DELETE-THIS";

    macro_rules! make_cmd {
        ($cmd: expr, $in:expr) => {
            $cmd.arg("cyclonedx")
                .arg("--output-prefix")
                .arg(OUTPUT_PREFIX)
                .arg("-f")
                .arg("json")
                .arg("--extended-metadata-path")
                .arg(format!("tests/ext_metadata/{}", $in));
        };
    }

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    make_cmd!(cmd, "1.3.json");

    cmd.assert().success();

    let input = "tests/ext_metadata/1.3.json";
    let input = std::fs::File::open(&input)
        .unwrap_or_else(|_| panic!("Failed to read input file: {:?}", input));
    let input = cyclonedx_bom::models::bom::Bom::parse_from_json_v1_3(input)?
        .metadata
        .unwrap();
    let output = std::fs::File::open(format!("{}.json", OUTPUT_PREFIX))
        .unwrap_or_else(|_| panic!("Failed to read input file: {}.json", OUTPUT_PREFIX));
    let output = cyclonedx_bom::models::bom::Bom::parse_from_json(output)?
        .metadata
        .unwrap();

    assert_eq!(input.authors, output.authors);
    assert_eq!(input.supplier, output.supplier);
    assert_eq!(
        input.component.unwrap().publisher,
        output.component.unwrap().publisher
    );

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    make_cmd!(cmd, "1.4.json");

    cmd.assert().success();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    make_cmd!(cmd, "1.5.json");

    cmd.assert().failure();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    make_cmd!(cmd, "1.3.xml");

    cmd.assert().success();

    let input = "tests/ext_metadata/1.3.xml";
    let input = std::fs::File::open(&input)
        .unwrap_or_else(|_| panic!("Failed to read input file: {:?}", input));
    let input = cyclonedx_bom::models::bom::Bom::parse_from_xml_v1_3(input)?
        .metadata
        .unwrap();
    let output = std::fs::File::open(format!("{}.json", OUTPUT_PREFIX))
        .unwrap_or_else(|_| panic!("Failed to read input file: {}.json", OUTPUT_PREFIX));
    let output = cyclonedx_bom::models::bom::Bom::parse_from_json(output)?
        .metadata
        .unwrap();

    assert_eq!(input.authors, output.authors);
    assert_eq!(input.supplier, output.supplier);
    assert_eq!(
        input.component.unwrap().publisher,
        output.component.unwrap().publisher
    );

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    make_cmd!(cmd, "1.4.xml");

    cmd.assert().success();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    make_cmd!(cmd, "1.5.xml");

    cmd.assert().failure();

    Ok(())
}
