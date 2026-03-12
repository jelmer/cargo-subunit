use anyhow::Result;
use chrono::Utc;
use std::io::Write;
use subunit::serialize::Serializable;
use subunit::types::event::Event;
use subunit::types::teststatus::TestStatus;

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
        let evt = match event {
            TestEvent::Started { name } => Event::new(TestStatus::InProgress)
                .test_id(name)
                .datetime(Utc::now())
                .map_err(|e| anyhow::anyhow!("Failed to create timestamp: {}", e))?
                .build(),
            TestEvent::Passed {
                name,
                duration_secs: _,
            } => Event::new(TestStatus::Success)
                .test_id(name)
                .datetime(Utc::now())
                .map_err(|e| anyhow::anyhow!("Failed to create timestamp: {}", e))?
                .build(),
            TestEvent::Failed {
                name,
                duration_secs: _,
                stdout,
                stderr,
            } => {
                let mut builder = Event::new(TestStatus::Failed)
                    .test_id(name)
                    .datetime(Utc::now())
                    .map_err(|e| anyhow::anyhow!("Failed to create timestamp: {}", e))?;

                // Note: subunit v2 allows only one file attachment per event
                // If both stdout and stderr exist, we prefer stderr (more important for failures)
                if let Some(stdout_content) = stdout {
                    if !stdout_content.is_empty() {
                        builder = builder
                            .mime_type("text/plain;charset=utf8")
                            .file_content("stdout", stdout_content.as_bytes());
                    }
                }

                if let Some(stderr_content) = stderr {
                    if !stderr_content.is_empty() {
                        builder = builder
                            .mime_type("text/plain;charset=utf8")
                            .file_content("stderr", stderr_content.as_bytes());
                    }
                }

                builder.build()
            }
            TestEvent::Ignored { name } => Event::new(TestStatus::Skipped)
                .test_id(name)
                .datetime(Utc::now())
                .map_err(|e| anyhow::anyhow!("Failed to create timestamp: {}", e))?
                .build(),
            TestEvent::Timeout {
                name,
                duration_secs: _,
            } => Event::new(TestStatus::Failed)
                .test_id(name)
                .datetime(Utc::now())
                .map_err(|e| anyhow::anyhow!("Failed to create timestamp: {}", e))?
                .mime_type("text/plain;charset=utf8")
                .file_content("reason", b"Test timed out")
                .build(),
        };

        evt.serialize(&mut self.output)
            .map_err(|e| anyhow::anyhow!("Failed to write subunit event: {}", e))?;

        // Flush after each event to ensure real-time output
        self.output.flush()?;

        Ok(())
    }

    /// Write a test existence event (for --list mode)
    pub fn write_test_exists(&mut self, test_name: &str) -> Result<()> {
        let evt = Event::new(TestStatus::Enumeration)
            .test_id(test_name)
            .datetime(Utc::now())
            .map_err(|e| anyhow::anyhow!("Failed to create timestamp: {}", e))?
            .build();

        evt.serialize(&mut self.output)
            .map_err(|e| anyhow::anyhow!("Failed to write subunit event: {}", e))?;

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
