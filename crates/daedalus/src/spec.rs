use crate::error::DaedalusError;
use serde::de::DeserializeOwned;

pub(crate) fn read<T: DeserializeOwned>(filename: &str) -> Result<T, DaedalusError> {
    read_yaml(filename.to_owned(), read_file(filename)?)
}

fn read_file(basename: &str) -> Result<String, DaedalusError> {
    let filename = shellexpand::tilde(basename).into_owned();
    std::fs::read_to_string(&filename)
        .map_err(|source| DaedalusError::NotFound { filename, source })
}

fn read_yaml<T: DeserializeOwned>(basename: String, string: String) -> Result<T, DaedalusError> {
    // from_str has its own Result with extra bounds, deconstruct and reconstruct here new Resust
    match serde_yaml::from_str(&string) {
        Ok(data) => Ok(data),
        Err(source) => Err(DaedalusError::Parsing { basename, source }),
    }
}
