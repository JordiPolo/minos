use crate::disparity::*;
use json::JsonValue;
use openapi;
use regex::*;

const UUID_STRING: &str =
    r"^(?i)[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[89AB][0-9A-F]{3}-[0-9A-F]{12}$";
const DATE_STRING: &str = r"^(?:[1-9]\d{3}-(?:(?:0[1-9]|1[0-2])-(?:0[1-9]|1\d|2[0-8])|(?:0[13-9]|1[0-2])-(?:29|30)|(?:0[13578]|1[02])-31)|(?:[1-9]\d(?:0[48]|[2468][048]|[13579][26])|(?:[2468][048]|[13579][26])00)-02-29)$";
const DATETIME_STRING: &str = r"^(?:[1-9]\d{3}-(?:(?:0[1-9]|1[0-2])-(?:0[1-9]|1\d|2[0-8])|(?:0[13-9]|1[0-2])-(?:29|30)|(?:0[13578]|1[02])-31)|(?:[1-9]\d(?:0[48]|[2468][048]|[13579][26])|(?:[2468][048]|[13579][26])00)-02-29)T(?:[01]\d|2[0-3]):[0-5]\d:[0-5]\d(?:Z|[+-][01]\d:[0-5]\d)$";

#[derive(Debug)]
pub struct StringValidator {
    response: String,
    format: StringFormat,
}

impl StringValidator {
    pub fn new(value: &JsonValue, schema: &openapi::v2::Schema) -> Self {
        match *value {
            // If we have a match, let's be more specific
            JsonValue::String(ref string) => StringValidator {
                response: string.to_owned(),
                format: StringFormat::new(schema),
            },
            JsonValue::Short(ref short) => StringValidator {
                response: short.as_str().to_owned(),
                format: StringFormat::new(schema),
            },
            // TODO: handle null case properly
            JsonValue::Null => StringValidator {
                response: "null".to_owned(),
                format: StringFormat::new(schema),
            },
            _ => {
                panic!(
                    "We type checked, It is not possible that the value {:?} from {:?} is not a string", value, schema
                );
            }
        }
    }
    pub fn validate(&self, location: &Location) -> Option<Disparity> {
        self.format.validate(&self.response, location)
    }
}

#[derive(Debug)]
enum StringFormat {
    Uuid,
    Date,
    DateTime,
    Unknown,
    None,
}

impl StringFormat {
    pub fn new(schema: &openapi::v2::Schema) -> Self {
        let format = schema.format.clone();
        match format {
            None => StringFormat::None,
            Some(string) => {
                if string == "uuid" {
                    StringFormat::Uuid
                } else if string == "date" {
                    StringFormat::Date
                } else if string == "date-time" {
                    StringFormat::DateTime
                } else {
                    StringFormat::Unknown
                    //panic!("Unknown string format {}", string)
                }
            }
        }
    }

    pub fn validate(&self, value: &str, location: &Location) -> Option<Disparity> {
        match *self {
            StringFormat::Uuid => {
                self.check_format_in_response(value, location, UUID_STRING, "UUID")
            }
            StringFormat::None => self.check_really_unknown_response(value, location),
            StringFormat::Date => {
                self.check_format_in_response(value, location, DATE_STRING, "date")
            }
            StringFormat::DateTime => {
                self.check_format_in_response(value, location, DATETIME_STRING, "date time")
            }
            StringFormat::Unknown => None,
        }
    }

    fn check_format_in_response(
        &self,
        value: &str,
        location: &Location,
        regex_str: &str,
        name: &str,
    ) -> Option<Disparity> {
        let uuid_regex = Regex::new(regex_str).unwrap();
        if uuid_regex.is_match(value) {
            None
        } else {
            Some(Disparity::new(
                &format!("The schema expects an {} but got {:?}", name, value),
                location.clone(),
            ))
        }
    }

    fn check_really_unknown_response(&self, value: &str, location: &Location) -> Option<Disparity> {
        let uuid_regex = Regex::new(UUID_STRING).unwrap();
        let datetime_regex = Regex::new(DATETIME_STRING).unwrap();
        let date_regex = Regex::new(DATE_STRING).unwrap();

        let error_string = if uuid_regex.is_match(value) {
            Some(self.unknown_error("uuid"))
        } else if datetime_regex.is_match(value) {
            Some(self.unknown_error("date-time"))
        } else if date_regex.is_match(value) {
            Some(self.unknown_error("date"))
        } else {
            None
        };

        error_string.map(|error| Disparity::new(&error, location.clone()))
    }

    fn unknown_error(&self, name: &str) -> String {
        format!(
            "Got a {} from the service but it is not properly defined in the OpenAPI file. Define it as type:string format:{}",
            name,
            name
        )
    }
}
