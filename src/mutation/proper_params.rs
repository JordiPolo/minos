use crate::known_param::KnownParamCollection;
use crate::request_param::RequestParam;
use chrono::prelude::*;
use openapi_utils::{IntegerTypeExt, ParameterDataExt, ParameterExt};
use openapiv3::Type;
use rand::seq::SliceRandom;
use std::ops::Range;

pub fn create_params(
    param: &openapiv3::Parameter,
    known_params: &KnownParamCollection,
) -> Option<RequestParam> {
    let name = param.name();
    if known_params.param_known(&name) {
        return Some(RequestParam::new(&name, &known_params.param_value(&name)));
    }
    if name == "page" || name == "per_page" || name == "include_count" {
        return pagination_param(&param);
    }
    match param.parameter_data().get_type() {
        Type::Boolean {} => boolean_request_param(&param),
        Type::Integer(_) => integer_request_param(&param),
        Type::String(openapiv3::StringType {
            format,
            enumeration,
            ..
        }) => string_request_param(&param, &format, &enumeration),
        _ => Some(RequestParam::new(&name, "truething")),
    }
}

// Get a valid param, if possible not the default one.
fn boolean_request_param(param: &openapiv3::Parameter) -> Option<RequestParam> {
    Some(RequestParam::new(param.name(), "false"))
    // TODO: find default
    // if param.clone().default.unwrap_or(true.into()) == true.into() {
    //     RequestParam::new(&param.name, "false")
    // } else {
    //     RequestParam::new(&param.name, "true")
    // }
}

fn integer_request_param(param: &openapiv3::Parameter) -> Option<RequestParam> {
    let default: i64 = 1; // TODO: param.clone().default.unwrap_or(1.into()).into();
    let minmax = limits(param);
    let min = minmax.start;
    let max = minmax.end;
    let mut value: i64 = (min + max) / 2;
    if value == default && value < max {
        value += 1;
    }
    Some(RequestParam::new(&param.name(), &format!("{:?}", value)))
}

fn string_request_param(
    param: &openapiv3::Parameter,
    format: &openapiv3::VariantOrUnknownOrEmpty<openapiv3::StringFormat>,
    enumeration: &Vec<String>,
) -> Option<RequestParam> {
    let name = param.parameter_data().name.clone();

    if !enumeration.is_empty() {
        let mut rng = rand::thread_rng();
        let value = enumeration.choose(&mut rng).unwrap();
        return Some(RequestParam::new(&name, value));
    }

    match format {
        openapiv3::VariantOrUnknownOrEmpty::Item(string_format) => match string_format {
            openapiv3::StringFormat::Date => Some(RequestParam::new(
                &name,
                &format!("{:?}", Utc.ymd(2019, 11, 28)),
            )),
            openapiv3::StringFormat::DateTime => {
                let date_time = Utc.ymd(2019, 11, 28).and_hms(12, 0, 9);
                Some(RequestParam::new(&name, &format!("{:?}", date_time)))
            }
            _ => unimplemented!("String format not supported"),
        },
        openapiv3::VariantOrUnknownOrEmpty::Unknown(string) => {
            if string == "uuid" {
                // We can't do a random uuid, as it will fail. None says we did not create a valid param
                None
            // let uuid = uuid::Uuid::new_v4();
            // RequestParam::new(&name, &format!("{:?}", uuid))
            // This is a bug in the openapiv3 library
            //https://github.com/glademiller/openapiv3/blob/master/src/schema.rs#L203
            } else if string == "date-time" {
                let date_time = Utc.ymd(2020, 1, 13).and_hms(12, 0, 9);
                Some(RequestParam::new(&name, &format!("{:?}", date_time)))
            } else {
                Some(RequestParam::new(&name, "PLAIN_STRING_UNKNOWN"))
                // TODO plain string
                // unimplemented!("No plain string support")
            }
        }
        openapiv3::VariantOrUnknownOrEmpty::Empty => {
            // TODO Better idea here
            Some(RequestParam::new(&name, "PLAIN_STRING_EMPTY"))
        }
    }
    // TODO: This where?
    // ProperParamsBuilder::to_string_enum_request_param(param)
}

// Proper pagination params as defined by Github.
fn pagination_param(param: &openapiv3::Parameter) -> Option<RequestParam> {
    let name = param.name();
    if name == "page" {
        Some(RequestParam::new(&name, "1"))
    } else if name == "per_page" {
        integer_request_param(&param)
    } else {
        // include_count
        Some(RequestParam::new(&name, "true"))
    }
}

fn limits(param: &openapiv3::Parameter) -> Range<i64> {
    match param.parameter_data().get_type() {
        Type::Integer(the_integer) => the_integer.limits(),
        Type::Number(_the_integer) => unimplemented!("Needs support for number"),
        _ => unimplemented!("Figure this out"),
    }
}
