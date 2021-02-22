use rand::seq::SliceRandom;
use serde::Deserialize;
use std::collections::BTreeMap;

use crate::error::DaedalusError;
use crate::spec;

// This is the values of the conversion
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
    pub(crate) fn param_value(&self, name: &str) -> Option<&String> {
        // We leave only paths that contain inside the param name we care about
        let matches = self
            .paths
            .iter()
            .filter(|(_name, path)| path.get(name).is_some());

        let mut the_default = None;
        let mut not_default = None;

        //b a_match is a tuple (string, Btreemap<string, StringOrArray>)
        for a_match in matches {
            if a_match.0 == &"/" {
                the_default = Some(a_match.1);
            } else {
                not_default = Some(a_match.1);
            }
        }

        // If posible choose a more specific value, it not the value under "/"" else we have None
        let result = not_default.or(the_default).or(None);

        result.and_then(|inner_hashmap| {
            inner_hashmap
                .get(name)
                .unwrap()
                .0
                .choose(&mut rand::thread_rng()) //choose returns an Option, we connect to it with and_then
        })
    }

    // a pattern may be /users/{uuid}/friends/{uuid2}
    pub(crate) fn retrieve_known_path(&self, pattern: &str) -> Option<String> {
        let mut r_default = None;
        let mut r_specific = None;

        for (path, inner_hash) in &self.paths {
            let mut result = pattern.to_owned();
            for (key, value) in *inner_hash {
                let random_value = &value.0.choose(&mut rand::thread_rng()).unwrap();
                // We want to accumulate the changes of result on itself so things with multiple variables like
                // /resource/{id}{tag} gets every variable replaced and saved
                result = str::replace(&result, &format!("{{{}}}", key), random_value)
            }
            // If we resolved all the variables
            if !result.contains('{') {
                if path == &&"/".to_string() {
                    r_default = Some(result);
                } else {
                    r_specific = Some(result);
                }
            }
        }
        r_specific.or(r_default).or(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testing_param_values() {
        let conversions =
            Conversions::new(&Some("tests/support/test_conversions.yaml".to_string())).unwrap();
        let all_matches = conversions.for_path("/");

        assert_eq!(
            all_matches.param_value("uuid"),
            Some(&"03b97130-1be2-42f9-bdaf-e1f6a2b9e269".to_string())
        );
        assert_eq!(all_matches.param_value("itisnothere"), None);

        // Retrieves a more specific uuid
        let user_matches = conversions.for_path("/users");
        assert_eq!(
            user_matches.param_value("uuid"),
            Some(&"11197130-1be2-42f9-bdaf-e1f6a2b9e111".to_string())
        );
    }

    #[test]
    fn testing_retrieve_known_paths() {
        let path = "/currencies/{uuid}";
        let conversions =
            Conversions::new(&Some("tests/support/test_conversions.yaml".to_string())).unwrap();
        let result = conversions.for_path(path).retrieve_known_path(path);
        assert_eq!(
            result,
            Some("/currencies/facaca04-d759-4d9d-99f5-fe97bd10a996".to_string())
        );
    }
}
