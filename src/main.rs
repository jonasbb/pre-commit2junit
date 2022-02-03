use color_eyre::eyre::{bail, eyre};
use color_eyre::Result;
use junit_report::{Duration, TestCase, TestCaseBuilder, TestSuite, OffsetDateTime};
use regex::Regex;
use std::io::{BufRead, Result as IoResult, Write};
use std::iter::Peekable;
use std::str::FromStr;

fn main() -> Result<()> {
    // Take one argument from the command line, the output file for the JUnit XML report.
    let mut args = std::env::args_os();
    // Skip program name
    args.next();

    let output_file = args
        .next()
        .ok_or_else(|| color_eyre::eyre::Report::msg("No output file on command line specified"))?;

    // Prepare inputs and outputs
    let stdin = std::io::stdin();
    let stdin = stdin.lock();
    let stderr = std::io::stderr();
    let mut stderr = stderr.lock();

    let mut testcase: Option<TestCase> = None;
    let mut testsuite = TestSuite::new("pre-commit");
    if std::env::var("CI").is_ok() {
        testsuite.hostname = String::from("localhost");
        testsuite.set_timestamp(OffsetDateTime::UNIX_EPOCH);
    }

    // Iterate over each line
    let mut stdin = stdin
        .lines()
        .map(|line| {
            line.and_then(|line| {
                writeln!(stderr, "{}", line)?;
                Ok(line)
            })
        })
        .peekable();
    let mut test_stdout = String::new();
    while let Some(line) = stdin.next() {
        let line = line?;

        let testcase_result = Regex::new(
            r"(?x)
            ^ # Start of line
            (?P<name>.+?) # Hook name
            (?:\.+) # A variable list of `.` for alignment
            (?P<reason>\(no\ files\ to\ check\))? # Optional reason why the hook was skipped
            (?:\x1b\[[0-9;]+m) # Optional color argument
            (?P<status>Passed|Failed|Skipped|Skipped) # Hook status
            (?:\x1b\[m)? # Optional color reset
            $ # End of line
            ",
        )?;
        if let Some(result) = testcase_result.captures(&line) {
            if let Some(mut testcase) = testcase.take() {
                testcase.set_system_out(&test_stdout);
                test_stdout.clear();
                testsuite.add_testcase(testcase);
            }

            let name = result
                .name("name")
                .ok_or(eyre!("Hook name not available"))?;
            let status = result
                .name("status")
                .ok_or(eyre!("Hook status not available"))?;

            let mut duration = Duration::ZERO;
            let mut hook_id = None;
            let mut exit_code = None;
            let mut modifies_files = false;

            /// Returns all lines which contain additional details, i.e., starting with `- `.
            fn additional_details<I>(input: &mut Peekable<I>) -> Option<IoResult<String>>
            where
                I: Iterator<Item = IoResult<String>>,
            {
                let peek = input.peek();
                let is_additional_details_line = peek.map_or(false, |line| {
                    line.as_ref().map_or(false, |line| {
                        line.starts_with("- ") || line.starts_with("\x1b[2m- ")
                    })
                });
                if is_additional_details_line {
                    input.next()
                } else {
                    None
                }
            }

            while let Some(additional) = additional_details(&mut stdin) {
                let additional = additional?;

                let additional = additional
                    .split_once("- ")
                    .expect("Must contain `- ` as checked by additional_details")
                    .1;
                // strip color suffix if available
                let additional = additional.strip_suffix("\x1b[m").unwrap_or(additional);

                // Process lines without a separator
                if additional == "files were modified by this hook" {
                    modifies_files = true;
                    continue;
                }

                let (name, value) = additional.split_once(": ").ok_or(eyre!(
                    "Must contain `: ` to be a valid additional details line"
                ))?;

                match name {
                    "hook id" => hook_id = Some(value.to_string()),
                    "exit code" => exit_code = Some(u32::from_str(value)?),
                    "duration" => {
                        // Strip trailing `s`
                        let value = value
                            .strip_suffix('s')
                            .ok_or(eyre!("Duration must end with `s`"))?;
                        let seconds = f64::from_str(value)?;
                        duration = Duration::seconds_f64(seconds);
                    }
                    _ => {}
                }
            }

            let mut builder = match status.as_str() {
                "Passed" => TestCaseBuilder::success(name.as_str(), duration),
                "Skipped" => TestCaseBuilder::skipped(name.as_str()),
                "Failed" => {
                    let type_;
                    let mut msg = String::new();

                    if let Some(exit_code) = exit_code {
                        type_ = String::from("Exit Code");
                        msg = format!("{}", exit_code);
                    } else if modifies_files {
                        type_ = String::from("Modified Files");
                    } else {
                        type_ = String::from("Unknown");
                    }

                    TestCaseBuilder::error(name.as_str(), duration, &type_, &msg)
                }
                status => bail!("Unknown status: {}", status),
            };

            if let Some(hook_id) = hook_id {
                builder.set_classname(&hook_id);
            }
            testcase = Some(builder.build());
        } else {
            // Does not match start of new
            test_stdout.push_str(&line);
            test_stdout.push('\n');
        }
    }

    // Add the last testcase
    if let Some(mut testcase) = testcase.take() {
        testcase.set_system_out(&test_stdout);
        test_stdout.clear();
        testsuite.add_testcase(testcase);
    }

    let report = junit_report::ReportBuilder::new()
        .add_testsuite(testsuite)
        .build();
    let mut junit_buffer = Vec::new();
    report.write_xml(&mut junit_buffer)?;
    std::fs::write(output_file, junit_buffer)?;
    Ok(())
}
