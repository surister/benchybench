use crate::benchmark::definition::{SessionValue, Statement, Suite};
use log::log;
use postgres::{Client, NoTls};
use std::error;
use std::fmt::format;
use std::io::Error;
use std::process::abort;
use std::time::{Duration, Instant};

pub(crate) struct BenchmarkRunner<R: Runner> {
    suites: Vec<Suite>,
    runner: R,
}

impl<R: Runner> BenchmarkRunner<R> {
    pub(crate) fn new(suites: Vec<Suite>, runner: R) -> Self {
        Self { suites, runner }
    }

    pub(crate) fn run(&mut self) {
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
    type Error;
    fn run_query(&mut self, statement: &String);

    fn prettify_error(&self, e: Self::Error) -> String;
}

pub(crate) struct PostgresRunner {
    client: Client,
}

impl PostgresRunner {
    pub(crate) fn new() -> Self {
        use postgres::{Client, NoTls};
        let client = Client::connect("host=localhost user=postgres", NoTls).unwrap();
        Self { client }
    }
}
impl Runner for PostgresRunner {
    type Error = postgres::error::Error;

    fn run_query(&mut self, statement: &String) {
        log::debug!("Running statement: {:}", statement);
        let result = self.client.query(statement, &[]);

        if let Err(e) = result {
            log::error!("statement: {:} produced the following error:", statement,);
            log::error!("{}", self.prettify_error(e));
            log::error!("aborting");

            abort()
        }
    }

    fn prettify_error(&self, e: Self::Error) -> String {
        use postgres::error::ErrorPosition;

        let Some(db) = e.as_db_error() else {
            // Connection errors, client-side errors, etc.
            return e.to_string();
        };

        // DbError's Display includes severity, message, detail, and hint.
        let mut output = format!("\n{db}");

        if let Some(context) = db.where_() {
            output.push_str(&format!("\nCONTEXT: {context}"));
        }
        match db.position() {
            Some(ErrorPosition::Original(position)) => {
                // do proper error marking in the future here.
                // output.push_str(format!("\n{}^", " ".repeat((position + 2) as usize)).as_str());
                // it will probably be easier if we also pass the offending query text to
                // the function and paint that with '^' given the error character position
                output.push_str(format!("\nPOSITION: character {position}").as_str());
            }
            Some(ErrorPosition::Internal { position, query }) => {
                output.push_str(&format!(
                    "\nINTERNAL QUERY: {query}\nPOSITION: character {position}"
                ));
            }
            None => {}
        }
        output.push_str(format!("\nSQLSTATE: {}", db.code().code()).as_str());
        output
    }
}
