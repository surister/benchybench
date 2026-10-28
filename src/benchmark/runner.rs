use crate::benchmark::definition::{Statement, Suite};
use std::time::{Duration, Instant};

pub(crate) struct BenchmarkRunner<R: Runner> {
    suites: Vec<Suite>,
    runner: R,
}

impl<R: Runner> BenchmarkRunner<R> {
    pub(crate) fn new(suites: Vec<Suite>, runner: R) -> Self {
        Self { suites, runner }
    }

    pub(crate) fn run(&self) {
        log::info!(
            "found {} suite(s): {:?}",
            self.suites.iter().count(),
            self.suites.iter().map(|s| &*s.name).collect::<Vec<&str>>()
        );

        // Requires resolution

        // Requires has already been resolved, suites here should be a correct
        // chains of benchmarks.
        for suite in self.suites.iter() {
            log::info!("running suite: {}", suite.name);
            // TODO: prepare-benchmark: run for every
            for benchmark in suite.benchmarks.iter() {
                let mut results: Vec<Duration> =
                    Vec::with_capacity(benchmark.benchmark_step.executions as usize);
                // TODO: session
                for _ in 0..benchmark.benchmark_step.executions {
                    // TODO: run before_run
                    let before = Instant::now();
                    {
                        for statement in benchmark.benchmark_step.steps.iter() {
                            self.runner.run_benchmark(statement)
                        }
                    }
                    results.push(before.elapsed())
                }
            }
            // TODO: teardown
        }
    }
}

pub(crate) trait Runner {
    fn run_benchmark(&self, statement: &Statement);
}

pub(crate) enum RunnerType {
    POSTGRES,
    HTTP,
}

#[derive(Debug)]
pub(crate) struct PostgresRunner {}
impl Runner for PostgresRunner {
    fn run_benchmark(&self, statement: &Statement) {
        log::info!("Running statement {:?}", self)
    }
}
