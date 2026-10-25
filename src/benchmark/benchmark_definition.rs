use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a Suite, a collection of `BenchmarkDefinition`s
struct Suite {
    name: String,
    benchmark_definition: Vec<BenchmarkDefinition>,
}

impl Suite {
    fn new(name: String, benchmarks: Vec<BenchmarkDefinition>) -> Suite {
        Suite {
            name,
            benchmark_definition: benchmarks,
        }
    }
}

/// Represents the entire benchmark definition. It provides all the necessary
/// fields to run a benchmark.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BenchmarkDefinition {
    /// The name of the benchmark.
    pub(crate) name: String,
    /// The version of the benchmark, sem–ver style.
    pub(crate) version: String,
    pub(crate) parameters: Option<HashMap<String, ParameterValue>>,
    pub(crate) provides: Option<Vec<String>>,
    #[serde(rename(serialize = "requires", deserialize = "requires"))]
    requires_step: Option<Vec<String>>,
    #[serde(rename(serialize = "benchmark", deserialize = "benchmark"))]
    benchmark_step: BenchmarkStep,
    #[serde(rename(serialize = "prepare-benchmark", deserialize = "prepare-benchmark"))]
    prepare_benchmark_step: Option<PrepareBenchmarkStep>,
    #[serde(rename(serialize = "teardown", deserialize = "teardown"))]
    teardown_step: Option<PrepareBenchmarkStep>,
}

/// Represents the benchmark step, e.g.
/// ```
/// benchmark: # <-------
///
///   # Sets a benchmark-wide session
///   session:
///     - SET timescaledb.enable_direct_compress_insert = false
/// ```
#[derive(Debug, Deserialize, Serialize)]
struct BenchmarkStep {
    /// The session parameters for the benchmark run, set for all runs.
    session: Vec<SessionValue>,
    /// The number of times the benchmark will run.
    executions: u8,
    /// The statements that will be benchmarked.
    steps: Vec<Statement>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PrepareBenchmarkStep {
    steps: Vec<Statement>,
}

/// Represents the possible value for a session parameter, e.g.
/// ```
/// benchmark:
///
///   # Sets a benchmark-wide session
///   session: # <-------
///     - SET timescaledb.enable_direct_compress_insert = false
/// ```
/// It would produce:
/// `SessionValue('SET timescaledb.enable_direct_compress_insert = false')`
///
/// It can only be a `String` or `None`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
enum SessionValue {
    String(String),
    None,
}

#[derive(Debug, Deserialize, Serialize)]
struct Statement {
    name: Option<String>,
    before_run: Option<String>,
    run: String,
    #[serde(default)]
    parallel: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum ParameterValue {
    Null(()),
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Vec(Vec<ParameterValue>),
    Map(HashMap<String, ParameterValue>),
}

pub(crate) fn definition_from_yaml(input: &str) -> Result<BenchmarkDefinition, serde_yaml::Error> {
    serde_yaml::from_str(input)
}

// we can easily add other format sources, serde automatically takes cares
// of mapping values to structs.
//
// pub(crate) fn definition_from_json(input: &str) -> Result<BenchmarkDefinition, serde_json::Error> {
//     serde_json::from_str(input)
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmark::benchmark_definition::ParameterValue::Vec;
    use serde::de::Unexpected::Str;
    use std::fs::File;
    use std::io::{Read, Write};

    #[test]
    fn test_can_parse_yaml() {
        let mut contents = String::new();
        File::open("../../benchmarks/benchy_definition.yml")
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        definition_from_yaml(&contents)
            .unwrap_or_else(|error| panic!("failed to parse benchmark YAML:\n{error:#?}"));
    }

    /// Test that a parsed benchy_definition.yml, the file that contains all keywords,
    /// contains every expected struct.
    #[test]
    fn test_has_all_fields_exists() {
        let mut contents = String::new();
        File::open("../../benchmarks/benchy_definition.yml")
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        let definition = definition_from_yaml(&contents).unwrap();
        // assert_eq!(definition.provides.unwrap(), vec![String::from("{p[table]}")]);
        // assert_eq!(definition.requires.unwrap(), vec!["some_table"]);
        // assert_eq!(definition.name, "Benchmark some very cool");
        // assert_eq!(definition.prepare_benchmark, "Benchmark some very cool");
        // assert_eq!(definition.parameters, vec!["s"])

        let file = File::create("t.json")
            .unwrap()
            .write(&*serde_json::to_string(&definition).unwrap().into_bytes());

        println!("{}", serde_json::to_string(&definition).unwrap());
    }
}
