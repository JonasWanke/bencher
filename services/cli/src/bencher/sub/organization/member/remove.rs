use std::convert::TryFrom;

use bencher_json::ResourceId;

use crate::{
    bencher::{backend::AuthBackend, sub::SubCmd},
    parser::organization::member::CliMemberRemove,
    CliError,
};

#[derive(Debug)]
pub struct Remove {
    pub org: ResourceId,
    pub user: ResourceId,
    pub backend: AuthBackend,
}

impl TryFrom<CliMemberRemove> for Remove {
    type Error = CliError;

    fn try_from(remove: CliMemberRemove) -> Result<Self, Self::Error> {
        let CliMemberRemove { org, user, backend } = remove;
        Ok(Self {
            org,
            user,
            backend: backend.try_into()?,
        })
    }
}

impl SubCmd for Remove {
    async fn exec(&self) -> Result<(), CliError> {
        let _json = self
            .backend
            .send(|client| async move {
                client
                    .org_member_delete()
                    .organization(self.org.clone())
                    .user(self.user.clone())
                    .send()
                    .await
            })
            .await?;
        Ok(())
    }
}
