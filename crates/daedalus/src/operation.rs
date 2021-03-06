#[derive(PartialEq, Clone, Debug)]
pub enum Crud {
    Index,
    Create,
    Show,
    Update,
    Delete,
    Patch,
}

impl Crud {
    // TODO This is kind of a hack to make mutator work with this piece
    // Probably we want to clean this up or just remove the concept of Crud
    // as it is only useful to tell the difference between index and show
    pub fn to_method_name(&self) -> &str {
        match self {
            Crud::Index => "GET",
            Crud::Show => "GET",
            Crud::Create => "POST",
            Crud::Update => "PUT",
            Crud::Patch => "PATCH",
            Crud::Delete => "DELETE",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Endpoint {
    pub crud: Crud,
    pub path_name: String,
    pub method: openapiv3::Operation,
}

impl Endpoint {
    fn new(crud: Crud, path_name: &str, method: openapiv3::Operation) -> Self {
        Endpoint {
            crud,
            path_name: path_name.to_string(),
            method,
        }
    }

    // TODO: Do not limit to GET
    pub(crate) fn new_supported(path_name: &str, methods: &openapiv3::PathItem) -> Vec<Self> {
        Self::create_supported_endpoint(path_name, methods)
            .into_iter()
            .filter(|x| x.crud == Crud::Show || x.crud == Crud::Index)
            .collect()
    }

    // TODO Return Vec which may be empty instead
    fn create_supported_endpoint(path_name: &str, methods: &openapiv3::PathItem) -> Vec<Self> {
        let mut vec = Vec::new();
        if Endpoint::url_with_variable(path_name) {
            let maybe_get = methods
                .get
                .clone()
                .map(|get| Endpoint::new(Crud::Show, path_name, get));
            let maybe_put = methods
                .put
                .clone()
                .map(|put| Endpoint::new(Crud::Update, path_name, put));
            let maybe_patch = methods
                .patch
                .clone()
                .map(|patch| Endpoint::new(Crud::Patch, path_name, patch));
            let maybe_delete = methods
                .delete
                .clone()
                .map(|delete| Endpoint::new(Crud::Delete, path_name, delete));

            vec.push(maybe_get);
            vec.push(maybe_put);
            vec.push(maybe_patch);
            vec.push(maybe_delete);
            vec.retain(|c| c.is_some());
            vec.into_iter().map(|o| o.unwrap()).collect()
        } else {
            let maybe_get = methods
                .get
                .clone()
                .map(|get| Endpoint::new(Crud::Index, path_name, get));
            let maybe_post = methods
                .put
                .clone()
                .map(|post| Endpoint::new(Crud::Create, path_name, post));
            vec.push(maybe_get);
            vec.push(maybe_post);
            vec.retain(|c| c.is_some());
            vec.into_iter().map(|o| o.unwrap()).collect()
        }
    }

    fn url_with_variable(path_name: &str) -> bool {
        path_name.contains('}')
    }
}
