use anyhow::{Context, Result};
use serde::Deserialize;

/// Events we care about from cargo test JSON output
#[derive(Debug, Clone)]
pub enum TestEvent {
    /// A test has started
    Started { name: String },
    /// A test passed
    Passed {
        name: String,
        #[allow(dead_code)]
        duration_secs: Option<f64>,
    },
    /// A test failed
    Failed {
        name: String,
        #[allow(dead_code)]
        duration_secs: Option<f64>,
        stdout: Option<String>,
        stderr: Option<String>,
    },
    /// A test was ignored/skipped
    Ignored { name: String },
    /// A test timed out
    Timeout {
        name: String,
        #[allow(dead_code)]
        duration_secs: Option<f64>,
    },
}

/// Top-level JSON event from cargo test
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum JsonEvent {
    Suite {
        #[allow(dead_code)]
        event: String,
        #[serde(default)]
        #[allow(dead_code)]
        test_count: Option<usize>,
    },
    Test {
        event: String,
        name: String,
        #[serde(default)]
        exec_time: Option<f64>,
        #[serde(default)]
        stdout: Option<String>,
        #[serde(default)]
        stderr: Option<String>,
    },
}

/// Parse a JSON line from cargo test output
pub fn parse_event(line: &str) -> Result<Option<TestEvent>> {
    let json_event: JsonEvent = serde_json::from_str(line).context("Failed to parse JSON event")?;

    match json_event {
        JsonEvent::Suite { .. } => {
            // We don't emit events for suite start/end
            Ok(None)
        }
        JsonEvent::Test {
            event,
            name,
            exec_time,
            stdout,
            stderr,
        } => {
            let test_event = match event.as_str() {
                "started" => TestEvent::Started { name },
                "ok" => TestEvent::Passed {
                    name,
                    duration_secs: exec_time,
                },
                "failed" => TestEvent::Failed {
                    name,
                    duration_secs: exec_time,
                    stdout,
                    stderr,
                },
                "ignored" => TestEvent::Ignored { name },
                "timeout" => TestEvent::Timeout {
                    name,
                    duration_secs: exec_time,
                },
                _ => {
                    // Unknown test event, skip
                    return Ok(None);
                }
            };
            Ok(Some(test_event))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_suite_started() {
        let line = r#"{"type":"suite","event":"started","test_count":3}"#;
        let event = parse_event(line).unwrap();
        assert!(event.is_none());
    }

    #[test]
    fn test_parse_test_started() {
        let line = r#"{"type":"test","event":"started","name":"my_test"}"#;
        let event = parse_event(line).unwrap().unwrap();
        match event {
            TestEvent::Started { name } => assert_eq!(name, "my_test"),
            _ => panic!("Expected Started event"),
        }
    }

    #[test]
    fn test_parse_test_passed() {
        let line = r#"{"type":"test","event":"ok","name":"my_test","exec_time":0.001}"#;
        let event = parse_event(line).unwrap().unwrap();
        match event {
            TestEvent::Passed {
                name,
                duration_secs,
            } => {
                assert_eq!(name, "my_test");
                assert_eq!(duration_secs, Some(0.001));
            }
            _ => panic!("Expected Passed event"),
        }
    }

    #[test]
    fn test_parse_test_failed() {
        let line = r#"{"type":"test","event":"failed","name":"my_test","exec_time":0.002,"stdout":"output","stderr":"error"}"#;
        let event = parse_event(line).unwrap().unwrap();
        match event {
            TestEvent::Failed {
                name,
                duration_secs,
                stdout,
                stderr,
            } => {
                assert_eq!(name, "my_test");
                assert_eq!(duration_secs, Some(0.002));
                assert_eq!(stdout, Some("output".to_string()));
                assert_eq!(stderr, Some("error".to_string()));
            }
            _ => panic!("Expected Failed event"),
        }
    }

    #[test]
    fn test_parse_test_ignored() {
        let line = r#"{"type":"test","event":"ignored","name":"my_test"}"#;
        let event = parse_event(line).unwrap().unwrap();
        match event {
            TestEvent::Ignored { name } => assert_eq!(name, "my_test"),
            _ => panic!("Expected Ignored event"),
        }
    }
}
