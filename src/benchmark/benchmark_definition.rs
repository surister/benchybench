use serde::{Deserialize, Serialize};
use std::collections::HashMap;
fn default_false_value() -> bool {
    false
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BenchmarkDefinition {
    pub(crate) name: String,
    pub(crate) parameters: Option<HashMap<String, ParameterValue>>,
    pub(crate) provides: Option<Vec<String>>,
    requires: Option<Vec<String>>,
    benchmark: Benchmark,
    #[serde(rename(serialize = "prepare-benchmark", deserialize = "prepare-benchmark"))]
    prepare_benchmark: Option<PrepareBenchmark>,
    teardown: Option<PrepareBenchmark>,
}
#[derive(Debug, Deserialize, Serialize)]
struct Benchmark {
    session: Vec<SessionValue>,
    executions: u8,
    steps: Vec<Statement>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PrepareBenchmark {
    steps: Vec<Statement>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
enum SessionValue {
    String(String),
    None,
}
#[derive(Debug, Deserialize, Serialize)]
struct Steps {
    statements: Vec<Statement>,
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
        File::open("benchmarks/bench.yml")
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        definition_from_yaml(&contents)
            .unwrap_or_else(|error| panic!("failed to parse benchmark YAML:\n{error:#?}"));
    }

    /// Test that a parsed bench.yml, the file that contains all keywords,
    /// contains every expected struct.
    #[test]
    fn test_has_all_fields_exists() {
        let mut contents = String::new();
        File::open("benchmarks/bench.yml")
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        let definition = definition_from_yaml(&contents).unwrap();
        assert_eq!(definition.provides.unwrap(), vec![String::from("{p[table]}")]);
        assert_eq!(definition.requires.unwrap(), vec!["some_table"]);
        assert_eq!(definition.name, "Benchmark some very cool");
        // assert_eq!(definition.prepare_benchmark, "Benchmark some very cool");
        // assert_eq!(definition.parameters, vec!["s"])

        let file = File::create("t.json")
            .unwrap()
            .write(&*serde_json::to_string(&definition).unwrap().into_bytes());

        println!("{}", serde_json::to_string(&definition).unwrap());
    }
}
