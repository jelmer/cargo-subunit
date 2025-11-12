use anyhow::Result;
use chrono::Utc;
use std::io::Write;
use subunit::Event;

use crate::json_parser::TestEvent;

/// Writer that converts test events to subunit format
pub struct SubunitWriter<W: Write> {
    output: W,
}

impl<W: Write> SubunitWriter<W> {
    /// Create a new subunit writer
    pub fn new(output: W) -> Self {
        Self { output }
    }

    /// Write a test event in subunit format
    pub fn write_event(&mut self, event: &TestEvent) -> Result<()> {
        match event {
            TestEvent::Started { name } => {
                let mut evt = Event {
                    status: Some("inprogress".to_string()),
                    test_id: Some(name.clone()),
                    timestamp: Some(Utc::now()),
                    file_name: None,
                    file_content: None,
                    mime_type: None,
                    route_code: None,
                    tags: None,
                };
                evt.write(&mut self.output)
                    .map_err(|e| anyhow::anyhow!("Failed to write subunit event: {}", e))?;
            }
            TestEvent::Passed {
                name,
                duration_secs: _,
            } => {
                let mut evt = Event {
                    status: Some("success".to_string()),
                    test_id: Some(name.clone()),
                    timestamp: Some(Utc::now()),
                    file_name: None,
                    file_content: None,
                    mime_type: None,
                    route_code: None,
                    tags: None,
                };
                evt.write(&mut self.output)
                    .map_err(|e| anyhow::anyhow!("Failed to write subunit event: {}", e))?;
            }
            TestEvent::Failed {
                name,
                duration_secs: _,
                stdout,
                stderr,
            } => {
                // First write the failure event
                let mut evt = Event {
                    status: Some("fail".to_string()),
                    test_id: Some(name.clone()),
                    timestamp: Some(Utc::now()),
                    file_name: None,
                    file_content: None,
                    mime_type: None,
                    route_code: None,
                    tags: None,
                };

                // Attach stdout if present
                if let Some(stdout_content) = stdout {
                    if !stdout_content.is_empty() {
                        evt.file_name = Some("stdout".to_string());
                        evt.file_content = Some(stdout_content.as_bytes().to_vec());
                        evt.mime_type = Some("text/plain;charset=utf8".to_string());
                    }
                }

                // Note: subunit v2 allows only one file attachment per event
                // If both stdout and stderr exist, we prefer stderr (more important for failures)
                if let Some(stderr_content) = stderr {
                    if !stderr_content.is_empty() {
                        evt.file_name = Some("stderr".to_string());
                        evt.file_content = Some(stderr_content.as_bytes().to_vec());
                        evt.mime_type = Some("text/plain;charset=utf8".to_string());
                    }
                }

                evt.write(&mut self.output)
                    .map_err(|e| anyhow::anyhow!("Failed to write subunit event: {}", e))?;
            }
            TestEvent::Ignored { name } => {
                let mut evt = Event {
                    status: Some("skip".to_string()),
                    test_id: Some(name.clone()),
                    timestamp: Some(Utc::now()),
                    file_name: None,
                    file_content: None,
                    mime_type: None,
                    route_code: None,
                    tags: None,
                };
                evt.write(&mut self.output)
                    .map_err(|e| anyhow::anyhow!("Failed to write subunit event: {}", e))?;
            }
            TestEvent::Timeout {
                name,
                duration_secs: _,
            } => {
                // Treat timeout as a failure
                let mut evt = Event {
                    status: Some("fail".to_string()),
                    test_id: Some(name.clone()),
                    timestamp: Some(Utc::now()),
                    file_name: Some("reason".to_string()),
                    file_content: Some(b"Test timed out".to_vec()),
                    mime_type: Some("text/plain;charset=utf8".to_string()),
                    route_code: None,
                    tags: None,
                };
                evt.write(&mut self.output)
                    .map_err(|e| anyhow::anyhow!("Failed to write subunit event: {}", e))?;
            }
        }

        // Flush after each event to ensure real-time output
        self.output.flush()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_started_event() {
        let mut output = Vec::new();
        let mut writer = SubunitWriter::new(&mut output);

        writer
            .write_event(&TestEvent::Started {
                name: "my_test".to_string(),
            })
            .unwrap();

        // Just verify that something was written
        assert!(!output.is_empty());
    }

    #[test]
    fn test_write_passed_event() {
        let mut output = Vec::new();
        let mut writer = SubunitWriter::new(&mut output);

        writer
            .write_event(&TestEvent::Passed {
                name: "my_test".to_string(),
                duration_secs: Some(0.5),
            })
            .unwrap();

        assert!(!output.is_empty());
    }

    #[test]
    fn test_write_failed_event() {
        let mut output = Vec::new();
        let mut writer = SubunitWriter::new(&mut output);

        writer
            .write_event(&TestEvent::Failed {
                name: "my_test".to_string(),
                duration_secs: Some(0.5),
                stdout: Some("test output".to_string()),
                stderr: Some("error message".to_string()),
            })
            .unwrap();

        assert!(!output.is_empty());
    }

    #[test]
    fn test_write_ignored_event() {
        let mut output = Vec::new();
        let mut writer = SubunitWriter::new(&mut output);

        writer
            .write_event(&TestEvent::Ignored {
                name: "my_test".to_string(),
            })
            .unwrap();

        assert!(!output.is_empty());
    }
}
