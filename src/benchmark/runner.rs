use crate::benchmark::definition::{BenchmarkDefinition, SessionValue, Statement, Suite};
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

/// Given a string parses interpolated values and returns a list of them.
///
/// Values are defined with brackets `'{'` `'}'`, every open bracket needs a closed bracket.
///
/// `parse_brackets('{hello}')` would produce `['hello']`
///
/// several values can be present, `'{table}_{n}'` is valid, and would produce `['table', 'n']`
///
/// nested values like `'{table_{n}}'` are not supported.
///
/// If open or closed brackets are found without their counterparts, it panics. The
/// algorithm is extremely simple and O(n), as every char is iterated only once.
fn parse_brackets(string: &String) -> Vec<String> {
    // we always return a vec of string and panic
    // if there is a wrong format, it's probably cleaner
    // to return an Option and make the caller raise the panics if Err.
    let mut left_bracket_i: Option<u8> = None;
    let mut params: Vec<String> = vec![];
    for (current_i, char) in string.char_indices() {
        if char == '{' {
            match left_bracket_i {
                None => left_bracket_i = Some(current_i as u8),
                // if a '{' is found, and it already found one, it's an unclosed '{'.
                Some(_) => panic!(
                    "found an open bracket without close bracket at char: {:?}",
                    current_i
                ),
            }
        }
        if char == '}' {
            match left_bracket_i {
                None => panic!("found unopened bracket at char: {:?}", current_i),
                Some(l_i) => {
                    // push the substring within left and right index.
                    params.push(string[l_i as usize + 1usize..current_i].to_string());
                    left_bracket_i = None;
                }
            }
        }
    }
    if let Some(i) = left_bracket_i {
        panic!("found open bracket without closed bracket at char: {:?}", i)
    }
    params
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

                if let Some(session) = &benchmark.benchmark_step.session {
                    session.iter().for_each(|value| {
                        if let SessionValue::String(query) = value {
                            self.runner.run_query(query);
                        }
                    });
                }

                for i in 0..benchmark.benchmark_step.executions {
                    log::info!("Run n. {}", i);
                    for statement in benchmark.benchmark_step.steps.iter() {
                        if let Some(query) = &statement.before_run {
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
                        .for_each(|query| self.runner.run_query(&query.run))
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

#[cfg(test)]
mod tests {
    use crate::benchmark::runner::parse_brackets;

    #[test]
    fn test_parsing_brackets() {
        assert_eq!(parse_brackets(&"{test1}".to_string()), vec!["test1"]);
        assert_eq!(parse_brackets(&"{test2}_2".to_string()), vec!["test2"]);
        assert_eq!(
            parse_brackets(&"{test3}_{test4}".to_string()),
            vec!["test3", "test4"]
        );
        assert_eq!(
            parse_brackets(&"test3_test4".to_string()),
            Vec::<String>::new()
        );
        assert_eq!(parse_brackets(&"".to_string()), Vec::<String>::new());
    }

    #[test]
    #[should_panic]
    fn test_parsing_missing_brackets_panics() {
        assert_eq!(parse_brackets(&"{test".to_string()), Vec::<String>::new());
        assert_eq!(parse_brackets(&"test{".to_string()), Vec::<String>::new());
        assert_eq!(parse_brackets(&"test}".to_string()), Vec::<String>::new());
        assert_eq!(
            parse_brackets(&"test{df}}}{{".to_string()),
            Vec::<String>::new()
        );
    }
}
