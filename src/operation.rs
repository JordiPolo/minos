use spec::Spec;

#[derive(PartialEq)]
pub enum CRUD {
    Index,
    Create,
    Show,
    Update,
    Delete,
    Patch,
}

pub struct Operation {
    pub crud: CRUD,
    pub method: openapi::v2::Operation,
}

impl Operation {
    pub fn new(crud: CRUD, method: openapi::v2::Operation) -> Self {
        Operation { crud, method }
    }

    pub fn understand_operation(
        _spec: &Spec,
        path_name: &str,
        methods: &openapi::v2::PathItem,
    ) -> Option<Operation> {
        // if Operation::looks_like_index(spec, path_name, methods) {
        //     Some(Operation::new(CRUD::Index, methods.clone().get.unwrap()))
        // } else { None }
        match Operation::url_ends_in_variable(path_name) {
            true => {
                let maybe_get = methods
                    .get
                    .clone()
                    .map(|get| Operation::new(CRUD::Show, get));
                let maybe_put = methods
                    .put
                    .clone()
                    .map(|put| Operation::new(CRUD::Update, put));
                let maybe_patch = methods
                    .patch
                    .clone()
                    .map(|patch| Operation::new(CRUD::Patch, patch));
                let maybe_delete = methods
                    .delete
                    .clone()
                    .map(|delete| Operation::new(CRUD::Delete, delete));

                maybe_get.or(maybe_put).or(maybe_patch).or(maybe_delete)
            }
            false => {
                let maybe_get = methods
                    .get
                    .clone()
                    .map(|get| Operation::new(CRUD::Index, get));
                let maybe_post = methods
                    .put
                    .clone()
                    .map(|post| Operation::new(CRUD::Create, post));
                maybe_get.or(maybe_post)
            }
        }
    }

    fn url_ends_in_variable(path_name: &str) -> bool {
        path_name.ends_with('}')
    }
}

// Keep it for the magic with params
//GET, no path params, no required query params
//     fn looks_like_index(spec: &Spec, path_name: &str, methods: &openapi::v2::PathItem) -> bool {
//         if Operation::url_ends_in_variable(path_name) {
//             return false;
//         }

//         match &methods.get {
//             Some(get) => {
//                 // Ok, so far so good.
//                 // Now we will not allow any required parameter. Maybe in the future we can improve on this
//                 let params_option = &get.parameters;
//                 match params_option {
//                     Some(params) => {
//                         params
//                             .iter()
//                             .all(|ref p| match request_params::resolve_parameter_ref(spec, p).required {
//                                 Some(required) => !required,
//                                 None => true,
//                             })
//                     }
//                     None => true,
//                 }
//             }
//             None => false,
//         }
//     }
// }

// Probably not needed
// fn has_url_params(path_name: &str) -> bool {
//     let re = Regex::new(r"^/[\w|-]+$").unwrap();
//     if re.is_match(path_name) {
//         false
//     } else {
//         true
//     }
// }
