// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Composable validation harness and pluggable output sinks.
//!
//! Absorbed from wetSpring V123 `Validator::finish_with_code()` and ludoSpring
//! V22 `ValidationSink` patterns. Provides structured, pluggable validation
//! output for `rhizocrypt doctor`, `rhizocrypt validate`, and any binary that
//! checks preconditions before proceeding.
//!
//! # Example
//!
//! ```
//! use rhizo_crypt_core::validation::{ValidationHarness, StringSink};
//!
//! let mut v = ValidationHarness::new("rhizocrypt doctor");
//! v.check("config file readable", true);
//! v.check("port available", true);
//! assert!(v.all_passed());
//!
//! let mut sink = StringSink::default();
//! let code = v.finish_to(&mut sink);
//! assert_eq!(code, 0);
//! assert!(sink.output.contains("2/2 checks passed"));
//! ```

/// Composable validation harness for binaries that check preconditions.
///
/// Unlike `OrExit`, this collects all failures before deciding the exit
/// code, making it suitable for `rhizocrypt doctor` and `rhizocrypt validate`.
///
/// # Example
///
/// ```
/// use rhizo_crypt_core::validation::ValidationHarness;
///
/// let mut v = ValidationHarness::new("rhizocrypt doctor");
/// v.check("config file readable", std::path::Path::new("/etc/hosts").exists());
/// v.check("port available", true);
/// assert!(v.all_passed());
/// // In a real binary: std::process::exit(v.exit_code().into());
/// ```
#[derive(Debug)]
pub struct ValidationHarness {
    label: String,
    checks: Vec<(String, bool)>,
}

impl ValidationHarness {
    /// Create a new validation harness.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            checks: Vec::new(),
        }
    }

    /// Record a named check result.
    pub fn check(&mut self, name: impl Into<String>, passed: bool) {
        self.checks.push((name.into(), passed));
    }

    /// Returns `true` if all checks passed.
    #[must_use]
    pub fn all_passed(&self) -> bool {
        self.checks.iter().all(|(_, passed)| *passed)
    }

    /// Number of checks that passed.
    #[must_use]
    pub fn pass_count(&self) -> usize {
        self.checks.iter().filter(|(_, p)| *p).count()
    }

    /// Number of checks that failed.
    #[must_use]
    pub fn fail_count(&self) -> usize {
        self.checks.iter().filter(|(_, p)| !*p).count()
    }

    /// Composable exit code: 0 if all passed, 1 otherwise.
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        u8::from(!self.all_passed())
    }

    /// Print a summary to stderr and return the exit code.
    ///
    /// Format:
    /// ```text
    /// [rhizocrypt doctor] 5/6 checks passed
    ///   ✓ config file readable
    ///   ✓ port available
    ///   ✗ discovery reachable
    /// ```
    #[must_use]
    pub fn finish(&self) -> u8 {
        self.finish_to(&mut StderrSink)
    }

    /// Print a summary to the given sink and return the exit code.
    ///
    /// Absorbed from ludoSpring V22 `ValidationSink` pattern. Enables
    /// pluggable output (JSON, file, buffer) for composable validation.
    #[must_use]
    pub fn finish_to(&self, sink: &mut dyn ValidationSink) -> u8 {
        let total = self.checks.len();
        let passed = self.pass_count();
        sink.header(&self.label, passed, total);
        for (name, ok) in &self.checks {
            sink.check(name, *ok);
        }
        self.exit_code()
    }

    /// Get the raw check results for programmatic access.
    #[must_use]
    pub fn checks(&self) -> &[(String, bool)] {
        &self.checks
    }
}

/// Pluggable output sink for [`ValidationHarness`].
///
/// Absorbed from ludoSpring V22. Implement this trait to redirect
/// validation output to JSON, files, or test buffers instead of stderr.
pub trait ValidationSink {
    /// Write the header line (e.g., `[label] 5/6 checks passed`).
    fn header(&mut self, label: &str, passed: usize, total: usize);
    /// Write a single check result line.
    fn check(&mut self, name: &str, passed: bool);
}

/// Default sink that writes to stderr.
pub struct StderrSink;

impl ValidationSink for StderrSink {
    fn header(&mut self, label: &str, passed: usize, total: usize) {
        eprintln!("[{label}] {passed}/{total} checks passed");
    }

    fn check(&mut self, name: &str, passed: bool) {
        let mark = if passed {
            "\u{2713}"
        } else {
            "\u{2717}"
        };
        eprintln!("  {mark} {name}");
    }
}

/// Sink that collects output into a `String` buffer (useful for testing).
#[derive(Debug, Default)]
pub struct StringSink {
    /// The collected output.
    pub output: String,
}

impl ValidationSink for StringSink {
    fn header(&mut self, label: &str, passed: usize, total: usize) {
        use std::fmt::Write;
        let _ = writeln!(self.output, "[{label}] {passed}/{total} checks passed");
    }

    fn check(&mut self, name: &str, passed: bool) {
        use std::fmt::Write;
        let mark = if passed {
            "\u{2713}"
        } else {
            "\u{2717}"
        };
        let _ = writeln!(self.output, "  {mark} {name}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_pass() {
        let mut v = ValidationHarness::new("test");
        v.check("check1", true);
        v.check("check2", true);
        assert!(v.all_passed());
        assert_eq!(v.pass_count(), 2);
        assert_eq!(v.fail_count(), 0);
        assert_eq!(v.exit_code(), 0);
    }

    #[test]
    fn with_failure() {
        let mut v = ValidationHarness::new("test");
        v.check("good", true);
        v.check("bad", false);
        v.check("also good", true);
        assert!(!v.all_passed());
        assert_eq!(v.pass_count(), 2);
        assert_eq!(v.fail_count(), 1);
        assert_eq!(v.exit_code(), 1);
    }

    #[test]
    fn empty_harness() {
        let v = ValidationHarness::new("empty");
        assert!(v.all_passed());
        assert_eq!(v.exit_code(), 0);
    }

    #[test]
    fn string_sink_output() {
        let mut v = ValidationHarness::new("sink_test");
        v.check("passes", true);
        v.check("fails", false);

        let mut sink = StringSink::default();
        let code = v.finish_to(&mut sink);
        assert_eq!(code, 1);
        assert!(sink.output.contains("[sink_test]"));
        assert!(sink.output.contains("1/2 checks passed"));
        assert!(sink.output.contains("\u{2713} passes"));
        assert!(sink.output.contains("\u{2717} fails"));
    }

    #[test]
    fn checks_accessor() {
        let mut v = ValidationHarness::new("accessor");
        v.check("a", true);
        v.check("b", false);
        let checks = v.checks();
        assert_eq!(checks.len(), 2);
        assert_eq!(checks[0], ("a".to_string(), true));
        assert_eq!(checks[1], ("b".to_string(), false));
    }
}
