use mauth_client::*;
use tracing::info;

pub struct Authentication {
    mauth_info: Option<mauth_client::MAuthInfo>
}

impl Authentication {
    pub fn new() -> Self {
        let auth = match MAuthInfo::from_default_file() {
            Err(_) => {
                info!("Mauth file not found on ~/.mauth_config.yml or incorrect format.");
                None
            },
            Ok(file) => Some(file)
        };
        Authentication {
            mauth_info: auth
        }
    }
    pub fn authenticate(&self, mut requ: &mut hyper::Request<hyper::Body>) {
        if let Some(mauth_info) = &self.mauth_info {
            // on empty body we digest "" TODO: Support request bodies
            let (_, body_digest) = MAuthInfo::build_body_with_digest("".to_string());
            mauth_info.sign_request_v2(&mut requ, &body_digest);
            mauth_info.sign_request_v1(&mut requ, &body_digest);
        }
    }
}
