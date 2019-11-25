#[derive(PartialEq, Clone)]
pub enum CRUD {
    Index,
    Create,
    Show,
    Update,
    Delete,
    Patch,
}

#[derive(Clone)]
pub struct Endpoint {
    pub crud: CRUD,
    pub path_name: String,
    pub method: openapiv3::Operation,
}

impl Endpoint {
    fn new(crud: CRUD, path_name: &str, method: openapiv3::Operation) -> Self {
        Endpoint {
            crud,
            path_name: path_name.to_string(),
            method,
        }
    }

    pub fn create_supported_endpoint(path_name: &str, methods: &openapiv3::PathItem) -> Option<Self> {
        if Endpoint::url_ends_in_variable(path_name) {
            let maybe_get = methods
                .get
                .clone()
                .map(|get| Endpoint::new(CRUD::Show, path_name, get));
            let maybe_put = methods
                .put
                .clone()
                .map(|put| Endpoint::new(CRUD::Update, path_name, put));
            let maybe_patch = methods
                .patch
                .clone()
                .map(|patch| Endpoint::new(CRUD::Patch, path_name, patch));
            let maybe_delete = methods
                .delete
                .clone()
                .map(|delete| Endpoint::new(CRUD::Delete, path_name, delete));

            maybe_get.or(maybe_put).or(maybe_patch).or(maybe_delete)
        } else {
            let maybe_get = methods
                .get
                .clone()
                .map(|get| Endpoint::new(CRUD::Index, path_name, get));
            let maybe_post = methods
                .put
                .clone()
                .map(|post| Endpoint::new(CRUD::Create, path_name, post));
            maybe_get.or(maybe_post)
        }
    }

    fn url_ends_in_variable(path_name: &str) -> bool {
        path_name.ends_with('}')
    }
}
