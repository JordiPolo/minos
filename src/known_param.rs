use serde::Deserialize;
use std::collections::BTreeMap;
use rand::seq::SliceRandom;

// StringOrArray and Raw to allow either strings or list of strings
#[derive(Deserialize, Debug, PartialEq)]
#[serde(from = "Raw")]
struct StringOrArray(Vec<String>);

impl From<Raw> for StringOrArray {
    fn from(raw: Raw) -> StringOrArray {
        match raw {
            Raw::String(s) => StringOrArray(vec![s]),
            Raw::List(l) => StringOrArray(l),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Raw {
    String(String),
    List(Vec<String>),
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Conversions {
    paths: BTreeMap<String, BTreeMap<String, StringOrArray>>,
}

impl Conversions {
    pub fn new(filename: &str) -> Self {
        match Self::_read(filename) {
            Err(_) => {
                println!(
                    "Conversions file {} not found. Conversions will not be used",
                    filename
                );
                Conversions {
                    paths: BTreeMap::new(),
                }
            }
            Ok(d) => d,
        }
    }

    fn _read(conversions: &str) -> Result<Conversions, std::io::Error> {
        let filename = shellexpand::tilde(conversions).into_owned();
        let filedata = std::fs::read_to_string(&filename)?;
        let deserialized_map: Conversions = serde_yaml::from_str(&filedata).expect(&format!(
            "The file {} could not be deserialized as a conversions YAML file.",
            conversions
        ));
        Ok(deserialized_map)
    }

    pub fn param_known(&self, name: &str) -> bool {
        self.paths
            .iter()
            .any(|(_name, path)| path.get(name).is_some())
    }

    pub fn param_value(&self, name: &str) -> String {
        let path = self.get_generic();
        let list =  &path.get(name).unwrap().0; // .0 to get the inner object from StringOrArray
        list.choose(&mut rand::thread_rng()).unwrap().to_string()
    }

    // a path may be /users/{uuid}/friends/{uuid2}
    pub fn retrieve_known_path(&self, pattern: &str) -> Option<String> {
        let mut result = String::new();

        for (path, keys) in &self.paths {
            if pattern.contains(path) {
                for (key, value) in keys {
                    let random_value = &value.0.choose(&mut rand::thread_rng()).unwrap();
                    result = str::replace(pattern, &format!("{{{}}}", key), random_value)
                }
                if !result.contains("{") {
                    return Some(result);
                }
            }
        }
        None
    }

    fn get_generic(&self) -> &BTreeMap<String, StringOrArray> {
        self.paths
            .iter()
            .find(|(name, _param)| **name == "/".to_string())
            .unwrap()
            .1
    }
}
