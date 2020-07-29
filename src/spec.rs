use tracing::debug;

pub fn read(filename: &str) -> openapiv3::OpenAPI {
    match std::fs::read_to_string(filename) {
        Ok(data) => {
            let spec = serde_yaml::from_str(&data)
                .expect("Could not deserialize file as OpenAPI v3.0 yaml");
            debug!("The openapi after parsed {:?}", spec);
            spec
        }
        Err(_) => {
            println!(
                "The openapi file in {} was not found. Please pass the option -f=<filename> with a valid filename.",
                filename
            );
            std::process::exit(-1);
        }
    }
}
