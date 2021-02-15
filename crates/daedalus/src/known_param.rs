use rand::seq::SliceRandom;
use serde::Deserialize;
use std::collections::BTreeMap;

use crate::error::DaedalusError;
use crate::spec;

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
    pub(crate) fn new(ofilename: &Option<String>) -> Result<Self, DaedalusError> {
        if let Some(filename) = ofilename {
            spec::read(filename)
        } else {
            Ok(Conversions {
                paths: BTreeMap::new(),
            })
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
        for (_path, keys) in self.paths.clone() {
            let mut result = pattern.to_owned();
            for (key, value) in keys {
                let random_value = &value.0.choose(&mut rand::thread_rng()).unwrap();
                // We want to accumulate the changes of result on itself so things with multiple variables like
                // /resource/{id}{tag} gets every variable replaced and saved
                result = str::replace(&result, &format!("{{{}}}", key), random_value)
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
