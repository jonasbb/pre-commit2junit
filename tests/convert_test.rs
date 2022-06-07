use assert_cmd::Command;
use pretty_assertions::assert_str_eq;

/// Test conversion of `pc-env1.fixture`
///
/// Create the fixture with `env SKIP=skipped pre-commit run --all-files --verbose`
#[test]
fn test_convert_fixture() {
    let mut cmd = Command::cargo_bin("pre-commit2junit").unwrap();
    let assert = cmd
        .arg("/dev/fd/1")
        .env("CI", "true")
        .write_stdin(include_str!("./pc-env1.fixture"))
        .assert();
    let assert = assert.success();
    let output = assert.get_output();
    let expected = r#"<?xml version="1.0" encoding="utf-8"?>
<testsuites>
  <testsuite id="0" name="pre-commit" package="testsuite/pre-commit" tests="6" errors="2" failures="0" hostname="localhost" timestamp="1970-01-01T00:00:00Z" time="1.399999999">
    <testcase name="Hook which always passes" time="0" classname="passing">
      <system-out><![CDATA[
success

]]></system-out>
    </testcase>
    <testcase name="Hook which always fails" time="0" classname="failing">
      <error type="Exit Code" message="1"><![CDATA[]]></error>
    </testcase>
    <testcase name="Slow hook for duration" time="1.399999999" classname="slow">
      <system-out><![CDATA[]]></system-out>
    </testcase>
    <testcase name="Hook does not run because no files" time="0" classname="missing-files">
      <skipped />
    </testcase>
    <testcase name="Hook skipped by environment" time="0" classname="skipped">
      <skipped />
    </testcase>
    <testcase name="Modifies files" time="0" classname="modifies">
      <error type="Modified Files" message=""><![CDATA[]]></error>
    </testcase>
  </testsuite>
</testsuites>"#;
    assert_str_eq!(expected, std::str::from_utf8(&output.stdout).unwrap());
}

/// Test conversion of `pc-env1-diff.fixture`
///
/// Create the fixture with `env SKIP=skipped pre-commit run --all-files --show-diff-on-failure --verbose`
#[test]
fn test_convert_fixture_with_diff() {
    let mut cmd = Command::cargo_bin("pre-commit2junit").unwrap();
    let assert = cmd
        .arg("/dev/fd/1")
        .env("CI", "true")
        .write_stdin(include_str!("./pc-env1-diff.fixture"))
        .assert();
    let assert = assert.success();
    let output = assert.get_output();
    let expected = r#"<?xml version="1.0" encoding="utf-8"?>
<testsuites>
  <testsuite id="0" name="pre-commit" package="testsuite/pre-commit" tests="6" errors="2" failures="0" hostname="localhost" timestamp="1970-01-01T00:00:00Z" time="1.399999999">
    <testcase name="Hook which always passes" time="0" classname="passing">
      <system-out><![CDATA[
success

]]></system-out>
    </testcase>
    <testcase name="Hook which always fails" time="0" classname="failing">
      <error type="Exit Code" message="1"><![CDATA[]]></error>
    </testcase>
    <testcase name="Slow hook for duration" time="1.399999999" classname="slow">
      <system-out><![CDATA[]]></system-out>
    </testcase>
    <testcase name="Hook does not run because no files" time="0" classname="missing-files">
      <skipped />
    </testcase>
    <testcase name="Hook skipped by environment" time="0" classname="skipped">
      <skipped />
    </testcase>
    <testcase name="Modifies files" time="0" classname="modifies">
      <error type="Modified Files" message=""><![CDATA[]]></error>
    </testcase>
    <system-out><![CDATA[pre-commit hook(s) made changes.
If you are seeing this message in CI, reproduce locally with: `pre-commit run --all-files`.
To run `pre-commit` as part of git workflow, use `pre-commit install`.
All changes made by hooks:
[1mdiff --git a/tests/pc-env1/changing_file.txt b/tests/pc-env1/changing_file.txt[m
[1mindex e61ef7b..5d308e1 100644[m
[1m--- a/tests/pc-env1/changing_file.txt[m
[1m+++ b/tests/pc-env1/changing_file.txt[m
[36m@@ -1 +1 @@[m
[31m-aa[m
[32m+[m[32maaaa[m
]]></system-out>
  </testsuite>
</testsuites>"#;
    assert_str_eq!(expected, std::str::from_utf8(&output.stdout).unwrap());
}
