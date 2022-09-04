use std::convert::TryFrom;

use async_trait::async_trait;
use bencher_json::JsonSignup;

use crate::{
    bencher::{
        backend::Backend,
        sub::SubCmd,
        wide::Wide,
    },
    cli::auth::CliAuthSignup,
    BencherError,
};

const SIGNUP_PATH: &str = "/v0/auth/signup";

#[derive(Debug, Clone)]
pub struct Signup {
    pub name:    String,
    pub slug:    Option<String>,
    pub email:   String,
    pub backend: Backend,
}

impl TryFrom<CliAuthSignup> for Signup {
    type Error = BencherError;

    fn try_from(signup: CliAuthSignup) -> Result<Self, Self::Error> {
        let CliAuthSignup {
            name,
            slug,
            email,
            host,
        } = signup;
        let backend = Backend::new(None, host)?;
        Ok(Self {
            name,
            slug,
            email,
            backend,
        })
    }
}

impl Into<JsonSignup> for Signup {
    fn into(self) -> JsonSignup {
        let Self {
            name,
            slug,
            email,
            backend: _,
        } = self;
        JsonSignup { name, slug, email }
    }
}

#[async_trait]
impl SubCmd for Signup {
    async fn exec(&self, _wide: &Wide) -> Result<(), BencherError> {
        let json_signup: JsonSignup = self.clone().into();
        let res = self.backend.post(SIGNUP_PATH, &json_signup).await?;
        let _: () = serde_json::from_value(res)?;
        Ok(())
    }
}
