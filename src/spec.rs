use openapi;
use std::string::String;

// TODO: This file needs a massive cleanup
#[derive(Clone, Debug)]
pub struct Spec {
    pub spec: openapi::v2::Spec,
}

impl Spec {
    pub fn json_ref_name(reference: &str) -> String {
        reference.split('/').last().unwrap().to_owned()
    }

    pub fn from_filename(filename: &str) -> Self {
        let spec = match openapi::from_path(filename).unwrap() {
            openapi::OpenApi::V2(spec) => spec,
            openapi::OpenApi::V3_0(_) => {
                panic!("{:?}", "Version 3.0 of the OpenAPI spec not supported")
            }
        };
        Spec { spec }
    }

    pub fn inline_everything(&mut self) {//-> Spec {
        let spec_clone = self.spec.clone();
        let def_clone = self.spec.definitions.clone().unwrap();

        self.inline_definitions();

        for (path_name, path_item) in self.spec.paths.iter_mut() {
            println!("{}", path_name);
            Spec::inline_operation(path_item.get.as_mut(), &spec_clone, &def_clone);
            Spec::inline_operation(path_item.post.as_mut(), &spec_clone, &def_clone);
            Spec::inline_operation(path_item.put.as_mut(), &spec_clone, &def_clone);
            Spec::inline_operation(path_item.patch.as_mut(), &spec_clone, &def_clone);
            Spec::inline_operation(path_item.delete.as_mut(), &spec_clone, &def_clone);
            Spec::inline_operation(path_item.options.as_mut(), &spec_clone, &def_clone);
            Spec::inline_operation(path_item.head.as_mut(), &spec_clone, &def_clone);
        }
    }

    fn inline_schema(schema: &mut openapi::v2::Schema, def_clone: &std::collections::BTreeMap<std::string::String, openapi::v2::Schema>) {
        if schema.ref_path.is_some() {
            let definition_name = schema.ref_path.as_ref().unwrap();
            *schema =  def_clone[&Spec::json_ref_name(&definition_name)].clone();
            Spec::inline_schema(schema, def_clone);
        }
        if schema.properties.is_some() {
            let properties = schema.properties.as_mut().unwrap();
            for (_property_name, property_schema) in properties.iter_mut() {
                if property_schema.ref_path.is_some() {
                    let definition_name = property_schema.ref_path.as_ref().unwrap();
                    *property_schema = def_clone[&Spec::json_ref_name(&definition_name)].clone();
                    Spec::inline_schema(property_schema, def_clone);
                }

                if property_schema.items.is_some() {
                    let items_schema = property_schema.items.as_mut().unwrap();
                    if items_schema.ref_path.is_some() {
                        let definition_name = items_schema.ref_path.as_ref().unwrap();
                        *items_schema = Box::new(def_clone[&Spec::json_ref_name(&definition_name)].clone());
                        Spec::inline_schema(items_schema, def_clone);
                    }
                }
            }
        }
        if schema.items.is_some() {
            let items_schema = schema.items.as_mut().unwrap();
            if items_schema.ref_path.is_some() {
                let definition_name = items_schema.ref_path.as_ref().unwrap();
                *items_schema = Box::new(def_clone[&Spec::json_ref_name(&definition_name)].clone());
                Spec::inline_schema(items_schema, def_clone);
            }
        }
    }

    fn inline_definitions(&mut self){//} -> openapi::v2::Spec{
        if self.spec.definitions.is_none() { return; }
        let def_clone = self.spec.definitions.clone().unwrap();
        let definitions = self.spec.definitions.as_mut().expect("Minos does not support files without definitions.");

        for (_name, schema) in definitions.into_iter() {
            Spec::inline_schema(schema, &def_clone);
        }
    }

    fn inline_operation(mut maybe_operation: Option<&mut openapi::v2::Operation>, spec_clone: &openapi::v2::Spec, def_clone: &std::collections::BTreeMap<std::string::String, openapi::v2::Schema>) {
        if maybe_operation.is_some() {
            let operation = maybe_operation.as_mut().unwrap();
            if operation.parameters.is_some() {
                let params = operation.parameters.as_mut().unwrap();
                for param in params {
                    *param = openapi::v2::ParameterOrRef::Parameter(Spec::resolve_parameter_ref(&spec_clone, &param));
                    match param {
                        openapi::v2::ParameterOrRef::Parameter(param) => {
                            if param.schema.is_some() {
                                Spec::inline_schema(param.schema.as_mut().unwrap(), &def_clone);
                            }
                        },
                        openapi::v2::ParameterOrRef::Ref { ref_path: _ } => (),
                    }
                }
            }
            for (name, response) in operation.responses.iter_mut() {
                if response.schema.is_some() {
                    println!("Response schema {}", name);
                    let response_schema = response.schema.as_mut().unwrap();
                    Spec::inline_schema(response_schema, &def_clone);
                }
            }
        }
    }



    pub fn resolve_parameter_ref3(
        param_or_ref: &openapi::v2::ParameterOrRef,
    ) -> openapi::v2::Parameter {
        match param_or_ref.clone() {
            openapi::v2::ParameterOrRef::Parameter(openapi::v2::Parameter {
                name,
                location,
                required,
                schema,
                unique_items,
                param_type,
                format,
                description,
                minimum,
                maximum,
                default,
                enum_values,
                exclusive_minimum,
                exclusive_maximum,
                max_length,
                min_length,
                max_items,
                min_items,
                pattern,
            }) => openapi::v2::Parameter {
                name,
                location,
                required,
                schema,
                unique_items,
                param_type,
                format,
                description,
                minimum,
                maximum,
                default,
                enum_values,
                exclusive_minimum,
                exclusive_maximum,
                max_length,
                min_length,
                max_items,
                min_items,
                pattern,
            },
            openapi::v2::ParameterOrRef::Ref { ref_path: _ } => panic!("this should not happen"),
        }
    }




    fn resolve_parameter(spec: &openapi::v2::Spec, parameter_name: &str) -> openapi::v2::Parameter {
        let global_params = spec.parameters.clone().unwrap();
        global_params[&Spec::json_ref_name(&parameter_name)].clone()
    }

    fn resolve_parameter_ref(
            spec: &openapi::v2::Spec,
        param_or_ref: &openapi::v2::ParameterOrRef,
    ) -> openapi::v2::Parameter {
        match param_or_ref.clone() {
            openapi::v2::ParameterOrRef::Parameter(openapi::v2::Parameter {
                name,
                location,
                required,
                schema,
                unique_items,
                param_type,
                format,
                description,
                minimum,
                maximum,
                default,
                enum_values,
                exclusive_minimum,
                exclusive_maximum,
                max_length,
                min_length,
                max_items,
                min_items,
                pattern,
            }) => openapi::v2::Parameter {
                name,
                location,
                required,
                schema,
                unique_items,
                param_type,
                format,
                description,
                minimum,
                maximum,
                default,
                enum_values,
                exclusive_minimum,
                exclusive_maximum,
                max_length,
                min_length,
                max_items,
                min_items,
                pattern,
            },
            openapi::v2::ParameterOrRef::Ref { ref_path } => Spec::resolve_parameter(spec, &ref_path),
        }
    }
}
