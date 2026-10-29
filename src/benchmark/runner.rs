use crate::benchmark::definition::{SessionValue, Statement, Suite};
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

            for benchmark in suite.benchmarks.iter() {
                if let Some(prepare_benchmark_step) = &benchmark.prepare_benchmark_step {
                    prepare_benchmark_step
                        .steps
                        .iter()
                        .filter(|x| x.run != "")
                        .map(|x| self.runner.run_query(&x.run))
                        .collect()
                }

                let mut results: Vec<Duration> =
                    Vec::with_capacity(benchmark.benchmark_step.executions as usize);
                for session in benchmark.benchmark_step.session.iter() {
                    match session {
                        SessionValue::String(query) => self.runner.run_query(query),
                        SessionValue::None => (),
                    }
                }
                for i in 0..benchmark.benchmark_step.executions {
                    log::info!("Run n. {}", i);
                    for statement in benchmark.benchmark_step.steps.iter() {
                        if let Some(query) = statement.before_run.as_ref() {
                            self.runner.run_query(&query);
                        }
                        let before = Instant::now();
                        {
                            self.runner.run_query(&statement.run)
                        }
                        results.push(before.elapsed())
                    }
                }
                if let Some(teardown) = &benchmark.teardown_step {
                    teardown
                        .steps
                        .iter()
                        .map(|query| self.runner.run_query(&query.run))
                        .collect()
                }
            }
        }
    }
}

pub(crate) trait Runner {
    fn run_query(&self, statement: &String);
}

#[derive(Debug)]
pub(crate) struct PostgresRunner {}
impl Runner for PostgresRunner {
    fn run_query(&self, statement: &String) {
        log::info!("Running statement {:?}", statement)
    }
}
