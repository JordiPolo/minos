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
    pub path_name: String,
    pub method: openapi::v2::Operation,
}

impl Operation {
    fn new(crud: CRUD, path_name: &str, method: openapi::v2::Operation) -> Self {
        Operation {
            crud,
            path_name: path_name.to_string(),
            method,
        }
    }

    pub fn create_supported_operation(path_name: &str, methods: &openapi::v2::PathItem) -> Option<Operation> {
        if Operation::url_ends_in_variable(path_name) {
            let maybe_get = methods
                .get
                .clone()
                .map(|get| Operation::new(CRUD::Show, path_name, get));
            let maybe_put = methods
                .put
                .clone()
                .map(|put| Operation::new(CRUD::Update, path_name, put));
            let maybe_patch = methods
                .patch
                .clone()
                .map(|patch| Operation::new(CRUD::Patch, path_name, patch));
            let maybe_delete = methods
                .delete
                .clone()
                .map(|delete| Operation::new(CRUD::Delete, path_name, delete));

            maybe_get.or(maybe_put).or(maybe_patch).or(maybe_delete)
        } else {
            let maybe_get = methods
                .get
                .clone()
                .map(|get| Operation::new(CRUD::Index, path_name, get));
            let maybe_post = methods
                .put
                .clone()
                .map(|post| Operation::new(CRUD::Create, path_name, post));
            maybe_get.or(maybe_post)
        }
    }

    fn url_ends_in_variable(path_name: &str) -> bool {
        path_name.ends_with('}')
    }
}
