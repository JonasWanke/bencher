use crate::parser::CliTypes;

use super::{swagger::Swagger, typeshare::Typeshare};

#[derive(Debug)]
pub struct Types {
    pub typeshare: Typeshare,
    pub swagger: Swagger,
}

impl TryFrom<CliTypes> for Types {
    type Error = anyhow::Error;

    fn try_from(_types: CliTypes) -> Result<Self, Self::Error> {
        Ok(Self {
            typeshare: Typeshare {},
            swagger: Swagger {},
        })
    }
}

impl Types {
    pub fn exec(&self) -> anyhow::Result<()> {
        self.typeshare.exec()?;
        self.swagger.exec()?;

        Ok(())
    }
}
