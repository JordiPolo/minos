use json::JsonValue;
use openapi;

use crate::checkers::*;
use crate::string_validator::*;
use crate::disparity::{Disparity, DisparityList, Location};


// This file is not needed anymore

pub struct Schema {
    pub schema: openapi::v2::Schema,
}

impl Schema {
    pub fn new(schema: openapi::v2::Schema) -> Self {
        Schema { schema }
    }

    // TODO: convert this to return errors and return errors instead of the early crappy return of disparities
    // TODO: many many clone()!
    // TODO: check for nulls
    pub fn validate(&self, response: &JsonValue, location: &Location) -> DisparityList {
        let schema = self.schema.clone();
        let mut disparities = DisparityList::new();

        if schema.schema_type.is_none() {
            println!(
                "We could not find a type at location {:?}. Types must always be specified in the OpenAPI file.",
                location
            );
            return disparities;
        }

        //TODO: match the option.
        let s_type = schema.schema_type.clone().unwrap();
        // println!("{:?} -> {:?}", location, s_type);

        // Incorrect type, fail here
        let type_disparity = check_response_type(response, &s_type, &location);
        if type_disparity.is_some() {
            disparities.option_push(type_disparity);
            return disparities;
        }

        // TODO: make an enum and a match instead of ifs
        if s_type == "array" {
            // This is an empty array because we already checked the type before
            if response.is_empty() {
                return disparities;
            }

            if schema.items.is_some() {
                let items = &schema.items.clone().unwrap();
                let new_location = location.clone().add("items");
                // TODO Support arrays of strings
                let new_schema = Schema::new(*items.clone());
                disparities
                    .merge(new_schema.validate(&response.members().as_slice()[0], &new_location));
            }

        } else if s_type == "object" {
            // Check that all the properties in the response are in the schema, and recurse on them
            let schema_properties = schema.properties.clone().unwrap().clone();
            for (property_name, property_value) in response.entries() {
                let property_schema = schema_properties.get(property_name);
                match property_schema {
                    Some(new_schema) => {
                        let rerew_schema = Schema::new(new_schema.clone());
                        disparities.merge(
                            rerew_schema.validate(property_value, &location.add(property_name)),
                        );
                    }
                    None => {
                        let error = Disparity::new(
                            &format!("Got a response with a property {:?} not described in your openapi file", property_name),
                            //TODO: improve location and simplify message
                            location.clone(),
                        );
                        disparities.push(error);
                    }
                }
            }
        // TODO: This works well, but do we really want to do it?
        // Check that the properties in the schema are there in the response. Don't need to recurse, done above.
        // for (schema_property_name, _schema_property_value) in schema_properties {
        //     if !response.has_key(&schema_property_name) {
        //             let error = Disparity::new(
        //                 &format!("The property {:?} described in your openapi file was not present in the real output.", schema_property_name),
        //                 //TODO: improve location and simplify message
        //                 location.clone(),
        //             );
        //             disparities.push(error);
        //     }
        // }
        } else if s_type == "string" {
            let validator = StringValidator::new(response, &schema);
            disparities.option_push(validator.validate(&location));
        } else if s_type == "number" || s_type == "integer" {
            //float and double
            disparities.option_push(check_number_format(response, &schema, &location));
        } else if s_type == "boolean" {
            //() // TODO: What do we need to check here?
        } else {
            panic!("Unknown type {:?}", s_type);
        }
        //         JsonValue::Boolean(boolean) => {},
        //         JsonValue::Null => {},

        disparities
    }
}
