#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

mod branch_name;
mod email;
mod error;
mod jwt;
mod non_empty;
mod resource_id;
mod slug;
mod user_name;

pub use crate::slug::Slug;
pub use branch_name::BranchName;
pub use email::Email;
pub use error::ValidError;
use error::REGEX_ERROR;
pub use jwt::Jwt;
pub use non_empty::NonEmpty;
pub use resource_id::ResourceId;
pub use user_name::UserName;

#[cfg(feature = "wasm")]
#[wasm_bindgen(start)]
pub fn startup() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
