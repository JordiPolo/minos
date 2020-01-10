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

    fn path_matches(&self, full_path: &str) -> bool {
        self.context == "path" && full_path.contains(&self.pattern)
    }

    fn query_matches(&self, query_param_name: &str) -> bool {
        self.context == "query" && query_param_name == self.pattern
    }
}

pub struct KnownParamCollection {
    params: Vec<KnownParam>,
}

impl KnownParamCollection {
    pub fn new() -> Self {
        KnownParamCollection {
            params: KnownParamCollection::read_known_params(),
        }
    }

    pub fn find_by_path(&self, path: &str) -> Option<KnownParam> {
        self.params
            .clone()
            .into_iter()
            .find(|param| param.path_matches(path))
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

    fn read_known_params() -> Vec<KnownParam> {
        match KnownParamCollection::_read_known_params() {
            Err(_) => vec![],
            Ok(d) => d,
        }
    }
    fn _read_known_params() -> Result<Vec<KnownParam>, std::io::Error> {
        let mut result = vec![];
        let filedata = std::fs::read_to_string("conversions.minos")?;
        for line in filedata.split('\n') {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 3 {
                result.push(KnownParam::new(&parts))
            }
        }
        Ok(result)
    }
}
