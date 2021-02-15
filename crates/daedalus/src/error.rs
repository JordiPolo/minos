use thiserror::Error;

#[derive(Error, Debug)]
pub enum DaedalusError {
    #[error("Could not deserialize the file `{basename}` as yaml.")]
    Parsing {
        basename: String,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("The Openapi file `{filename}` was not found or could not be read.")]
    NotFound {
        filename: String,
        #[source]
        source: std::io::Error,
    },
}
