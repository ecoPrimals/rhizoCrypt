// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Chaos testing framework — structured fault injection for resilience testing.
//!
//! Absorbed from squirrel's `ChaosEngine` pattern, adapted for `rhizoCrypt`'s
//! DAG-centric workload. The engine runs a configurable set of chaos scenarios
//! and collects metrics on how the system behaves under stress.
//!
//! ## Architecture
//!
//! ```text
//! ChaosConfig   → defines which fault classes to inject
//! ChaosEngine   → executes scenarios, collects metrics
//! ChaosScenario → individual fault injection test case
//! ChaosMetrics  → aggregated results across all scenarios
//! ```

use std::fmt;
use std::time::{Duration, Instant};

/// Fault injection classes that the chaos engine can simulate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FaultClass {
    /// Network partition (IPC socket unreachable).
    NetworkPartition,
    /// Slow responses (injected latency).
    Latency,
    /// Process crash / restart.
    ProcessCrash,
    /// Resource exhaustion (memory, file descriptors).
    ResourceExhaustion,
    /// Clock skew / time anomalies.
    ClockSkew,
    /// Concurrent mutation storm.
    ConcurrencyStorm,
    /// Corrupt input data.
    CorruptInput,
}

impl fmt::Display for FaultClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NetworkPartition => write!(f, "network_partition"),
            Self::Latency => write!(f, "latency"),
            Self::ProcessCrash => write!(f, "process_crash"),
            Self::ResourceExhaustion => write!(f, "resource_exhaustion"),
            Self::ClockSkew => write!(f, "clock_skew"),
            Self::ConcurrencyStorm => write!(f, "concurrency_storm"),
            Self::CorruptInput => write!(f, "corrupt_input"),
        }
    }
}

/// Configuration for a chaos testing run.
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    /// Which fault classes to inject.
    pub fault_classes: Vec<FaultClass>,
    /// Maximum duration for the entire chaos run.
    pub max_duration: Duration,
    /// Number of iterations per scenario.
    pub iterations_per_scenario: u32,
    /// Injected latency range for Latency faults.
    pub latency_range: (Duration, Duration),
    /// Concurrent task count for `ConcurrencyStorm`.
    pub concurrency_level: u32,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            fault_classes: vec![
                FaultClass::NetworkPartition,
                FaultClass::Latency,
                FaultClass::ConcurrencyStorm,
                FaultClass::CorruptInput,
            ],
            max_duration: Duration::from_secs(60),
            iterations_per_scenario: 10,
            latency_range: (Duration::from_millis(50), Duration::from_millis(500)),
            concurrency_level: 16,
        }
    }
}

/// Result of a single chaos scenario execution.
#[derive(Debug, Clone)]
pub struct ScenarioResult {
    /// Name of the scenario.
    pub name: String,
    /// Fault class used.
    pub fault_class: FaultClass,
    /// Whether the system survived (no panic, no corruption).
    pub survived: bool,
    /// Duration the scenario took.
    pub duration: Duration,
    /// Number of errors observed (expected under chaos).
    pub errors_observed: u32,
    /// Number of successful recoveries.
    pub recoveries: u32,
    /// Optional notes from the scenario.
    pub notes: Option<String>,
}

/// A chaos scenario that can be executed by the engine.
pub trait ChaosScenario: Send + Sync {
    /// Human-readable name for this scenario.
    fn name(&self) -> &str;

    /// Which fault class this scenario injects.
    fn fault_class(&self) -> FaultClass;

    /// Execute the scenario, returning a result.
    fn execute(&self, config: &ChaosConfig) -> ScenarioResult;
}

/// Aggregated metrics from a chaos testing run.
#[derive(Debug, Clone)]
pub struct ChaosMetrics {
    /// Total scenarios executed.
    pub total_scenarios: u32,
    /// Scenarios where the system survived.
    pub survived: u32,
    /// Scenarios where the system failed.
    pub failed: u32,
    /// Total errors observed across all scenarios.
    pub total_errors: u32,
    /// Total recoveries across all scenarios.
    pub total_recoveries: u32,
    /// Total wall-clock time for the entire run.
    pub total_duration: Duration,
    /// Individual scenario results.
    pub results: Vec<ScenarioResult>,
}

impl ChaosMetrics {
    /// Survival rate as a fraction (0.0 to 1.0).
    #[must_use]
    pub fn survival_rate(&self) -> f64 {
        if self.total_scenarios == 0 {
            return 1.0;
        }
        f64::from(self.survived) / f64::from(self.total_scenarios)
    }

    /// Pretty-print the metrics summary.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "Chaos run: {}/{} survived ({:.1}%), {} errors, {} recoveries, {:.1}s",
            self.survived,
            self.total_scenarios,
            self.survival_rate() * 100.0,
            self.total_errors,
            self.total_recoveries,
            self.total_duration.as_secs_f64(),
        )
    }
}

/// The chaos testing engine.
///
/// Registers scenarios, executes them according to configuration, and
/// collects metrics on system resilience.
pub struct ChaosEngine {
    config: ChaosConfig,
    scenarios: Vec<Box<dyn ChaosScenario>>,
}

impl ChaosEngine {
    /// Create a new engine with the given configuration.
    #[must_use]
    pub fn new(config: ChaosConfig) -> Self {
        Self {
            config,
            scenarios: Vec::new(),
        }
    }

    /// Create with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(ChaosConfig::default())
    }

    /// Register a scenario.
    pub fn add_scenario(&mut self, scenario: impl ChaosScenario + 'static) {
        self.scenarios.push(Box::new(scenario));
    }

    /// Execute all registered scenarios and return aggregated metrics.
    #[must_use]
    pub fn run(&self) -> ChaosMetrics {
        let start = Instant::now();
        let mut results = Vec::new();

        for scenario in &self.scenarios {
            if start.elapsed() >= self.config.max_duration {
                break;
            }

            if !self.config.fault_classes.contains(&scenario.fault_class()) {
                continue;
            }

            let result = scenario.execute(&self.config);
            results.push(result);
        }

        let survived =
            u32::try_from(results.iter().filter(|r| r.survived).count()).unwrap_or(u32::MAX);
        let failed =
            u32::try_from(results.iter().filter(|r| !r.survived).count()).unwrap_or(u32::MAX);
        let total_errors = results.iter().map(|r| r.errors_observed).sum();
        let total_recoveries = results.iter().map(|r| r.recoveries).sum();

        ChaosMetrics {
            total_scenarios: u32::try_from(results.len()).unwrap_or(u32::MAX),
            survived,
            failed,
            total_errors,
            total_recoveries,
            total_duration: start.elapsed(),
            results,
        }
    }

    /// Access the configuration.
    #[must_use]
    pub const fn config(&self) -> &ChaosConfig {
        &self.config
    }
}

impl fmt::Debug for ChaosEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChaosEngine")
            .field("config", &self.config)
            .field("scenarios", &self.scenarios.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyScenario {
        name: String,
        fault: FaultClass,
        survive: bool,
    }

    impl ChaosScenario for DummyScenario {
        fn name(&self) -> &str {
            &self.name
        }
        fn fault_class(&self) -> FaultClass {
            self.fault
        }
        fn execute(&self, _config: &ChaosConfig) -> ScenarioResult {
            ScenarioResult {
                name: self.name.clone(),
                fault_class: self.fault,
                survived: self.survive,
                duration: Duration::from_millis(10),
                errors_observed: u32::from(!self.survive),
                recoveries: u32::from(self.survive),
                notes: None,
            }
        }
    }

    #[test]
    fn empty_engine_runs() {
        let engine = ChaosEngine::with_defaults();
        let metrics = engine.run();
        assert_eq!(metrics.total_scenarios, 0);
        assert!((metrics.survival_rate() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn engine_runs_registered_scenarios() {
        let mut engine = ChaosEngine::with_defaults();
        engine.add_scenario(DummyScenario {
            name: "net_test".into(),
            fault: FaultClass::NetworkPartition,
            survive: true,
        });
        engine.add_scenario(DummyScenario {
            name: "latency_test".into(),
            fault: FaultClass::Latency,
            survive: false,
        });

        let metrics = engine.run();
        assert_eq!(metrics.total_scenarios, 2);
        assert_eq!(metrics.survived, 1);
        assert_eq!(metrics.failed, 1);
        assert!((metrics.survival_rate() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn engine_skips_disabled_fault_classes() {
        let config = ChaosConfig {
            fault_classes: vec![FaultClass::Latency],
            ..ChaosConfig::default()
        };
        let mut engine = ChaosEngine::new(config);
        engine.add_scenario(DummyScenario {
            name: "network".into(),
            fault: FaultClass::NetworkPartition,
            survive: true,
        });
        engine.add_scenario(DummyScenario {
            name: "latency".into(),
            fault: FaultClass::Latency,
            survive: true,
        });

        let metrics = engine.run();
        assert_eq!(metrics.total_scenarios, 1);
        assert_eq!(metrics.results[0].name, "latency");
    }

    #[test]
    fn metrics_summary_format() {
        let metrics = ChaosMetrics {
            total_scenarios: 10,
            survived: 8,
            failed: 2,
            total_errors: 5,
            total_recoveries: 7,
            total_duration: Duration::from_millis(1234),
            results: Vec::new(),
        };
        let s = metrics.summary();
        assert!(s.contains("8/10 survived"));
        assert!(s.contains("80.0%"));
    }

    #[test]
    fn fault_class_display() {
        assert_eq!(FaultClass::NetworkPartition.to_string(), "network_partition");
        assert_eq!(FaultClass::ClockSkew.to_string(), "clock_skew");
    }

    #[test]
    fn default_config_sensible() {
        let cfg = ChaosConfig::default();
        assert!(!cfg.fault_classes.is_empty());
        assert!(cfg.iterations_per_scenario > 0);
        assert!(cfg.concurrency_level > 0);
    }
}
