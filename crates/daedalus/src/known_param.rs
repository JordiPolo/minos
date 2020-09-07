use rand::seq::SliceRandom;
use serde::Deserialize;
use std::collections::BTreeMap;

// StringOrArray and Raw to allow either strings or list of strings
#[derive(Deserialize, Debug, PartialEq, Clone)]
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
pub(crate) struct Conversions {
    paths: BTreeMap<String, BTreeMap<String, StringOrArray>>,
}

impl Conversions {
    pub(crate) fn new(filename: &str) -> Self {
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

    pub(crate) fn for_path<'a>(&'a self, pattern: &str) -> ConversionView {
        let mut result = BTreeMap::new();
        for (path, keys) in &self.paths {
            if pattern.contains(path) {
                result.insert(path, keys);
            }
        }
        ConversionView { paths: result }
    }

    fn _read(conversions: &str) -> Result<Conversions, std::io::Error> {
        let filename = shellexpand::tilde(conversions).into_owned();
        let filedata = std::fs::read_to_string(&filename)?;
        match serde_yaml::from_str(&filedata) {
            Err(_) => panic!(
                "The file {} could not be deserialized as a conversions YAML file.",
                conversions
            ),
            Ok(deserialized_map) => Ok(deserialized_map),
        }
    }
}

// This is a view of known parameters for a particular endpoint (path)
// This is from where we can retrieve known parameters
#[derive(Debug, PartialEq)]
pub(crate) struct ConversionView<'a> {
    paths: BTreeMap<&'a String, &'a BTreeMap<String, StringOrArray>>,
}

impl<'a> ConversionView<'a> {
    pub(crate) fn param_value(&self, name: &str) -> Option<String> {
        let mut matches = self.matches(name);

        if matches.is_empty() {
            return None;
        }

        // If possible, we get a more specific value
        if matches.len() > 1 {
            matches.remove(&"/".to_string());
        }
        // Get the name from the first element, not sure if there is a better way to find that
        let values: Vec<&&BTreeMap<String, StringOrArray>> = matches.values().collect();
        let value_list = &values[0].get(name).unwrap().0;

        Some(
            value_list
                .choose(&mut rand::thread_rng())
                .unwrap()
                .to_string(),
        )
    }

    // a pattern may be /users/{uuid}/friends/{uuid2}
    // TODO: use the logic above to use non "/" if possible
    pub(crate) fn retrieve_known_path(&self, pattern: &str) -> Option<String> {
        let mut result = String::new();

        for (path, keys) in self.paths.clone() {
            for (key, value) in keys {
                let random_value = &value.0.choose(&mut rand::thread_rng()).unwrap();
                result = str::replace(pattern, &format!("{{{}}}", key), random_value)
            }
            if !result.contains('{') {
                return Some(result);
            }
        }
        None
    }

    // TODO: not clone
    fn matches(&self, name: &str) -> BTreeMap<&String, &BTreeMap<String, StringOrArray>> {
        self.paths
            .clone()
            .into_iter()
            .filter(|(_name, path)| path.get(name).is_some())
            .collect()
    }
}
