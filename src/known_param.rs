#[derive(Debug, Clone)]
pub struct KnownParam {
    pub pattern: String,
    pub value: String,
    pub context: String,
}

impl KnownParam {
    fn new(data: &[&str]) -> Self {
        KnownParam {
            pattern: data[1].to_string(),
            value: data[2].to_string(),
            context: data[0].to_string(),
        }
    }

    // We want:
    // pattern {user_uuid} path: /v1/users/{user_uuid} -> true
    // pattern {user_uuid} path: /v1/users/{user_uuid}/onboard -> true
    // pattern {user_uuid} path: /v1/users/{user_uuid}/friends/{friend_uuid} -> false
    fn path_matches(&self, full_path: &str) -> bool {
        self.context == "path"
            && full_path.contains(&self.pattern)
            && full_path.matches('}').count() == self.pattern.matches('}').count()
    }

    fn query_matches(&self, query_param_name: &str) -> bool {
        self.context == "query" && query_param_name == self.pattern
    }
}

pub struct KnownParamCollection {
    params: Vec<KnownParam>,
}

impl KnownParamCollection {
    pub fn new(conversions: &str) -> Self {
        KnownParamCollection {
            params: KnownParamCollection::read_known_params(conversions),
        }
    }

    // a path may be /users/{uuid}/friends/{uuid2}
    pub fn retrieve_known_path(&self, path: &str) -> Option<String> {
        let conversion = self.find_by_path(path)?;
        Some(str::replace(path, &conversion.pattern, &conversion.value))
    }

    pub fn param_known(&self, name: &str) -> bool {
        self.params.iter().any(|param| param.query_matches(name))
    }

    pub fn param_value(&self, name: &str) -> String {
        self.params
            .iter()
            .find(|param| param.query_matches(name))
            .unwrap()
            .value
            .to_string()
    }

    fn find_by_path(&self, path: &str) -> Option<KnownParam> {
        self.params
            .clone()
            .into_iter()
            .find(|param| param.path_matches(path))
    }

    fn read_known_params(conversions: &str) -> Vec<KnownParam> {
        match KnownParamCollection::_read_known_params(conversions) {
            Err(_) => {
                println!(
                    "Conversions file {} not found. Conversions will not be used",
                    conversions
                );
                vec![]
            }
            Ok(d) => d,
        }
    }
    fn _read_known_params(conversions: &str) -> Result<Vec<KnownParam>, std::io::Error> {
        let filename = shellexpand::tilde(conversions).into_owned();
        let filedata = std::fs::read_to_string(&filename)?;

        Ok(filedata.split('\n').fold(vec![], |mut acc, line| {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 3 {
                acc.push(KnownParam::new(&parts))
            }
            acc
        }))
    }
}
