use serde::{Deserialize, Serialize};

mod product;

pub(super) use product::UpstreamProduct;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UserInResponse {
    pub token: String,
}
