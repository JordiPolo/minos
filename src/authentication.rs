//use mauth_client::*;

// pub struct Authentication {
//     mauth_info: mauth_client::MAuthInfo
// }

// impl Authentication {
//     pub fn new() -> Self {
//         Authentication {
//             mauth_info: MAuthInfo::from_default_file().expect("Mauth file missing")
//         }
//     }
//     pub fn authenticate(&self, mut requ: &mut hyper::Request<hyper::Body>) {
//         // on empty body we digest "" TODO: Support request bodies
//         let (_, body_digest) = MAuthInfo::build_body_with_digest("".to_string());
//         self.mauth_info.sign_request_v2(&mut requ, &body_digest);
//         self.mauth_info.sign_request_v1(&mut requ, &body_digest);
//     }
// }

pub struct Authentication {}

impl Authentication {
    pub fn new() -> Self {
        Authentication {}
    }
    pub fn authenticate(&self, mut _requ: &mut hyper::Request<hyper::Body>) {}
}
