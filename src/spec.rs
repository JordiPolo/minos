use serde_yaml;
use std::path::Path;

pub fn read<P: AsRef<Path>>(filename: P) -> openapiv3::OpenAPI {
    let data = std::fs::read_to_string(filename).expect("OpenAPI file could not be read.");
    let spec =
        serde_yaml::from_str(&data).expect("Could not deserialize file as OpenAPI v3.0 yaml");
//    debug!("The openapi after parsed {:?}", spec);
    spec
}
