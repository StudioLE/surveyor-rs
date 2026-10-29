//! [`GetVersionTags`] stub.

use crate::prelude::*;

/// [`GetVersionTags`] stub.
pub struct GetVersionTagsStub {
    /// Version tags of the repo.
    ///
    /// Default: [`VersionTags::fixture`]
    pub tags: VersionTags,
}

impl Default for GetVersionTagsStub {
    fn default() -> Self {
        Self {
            tags: VersionTags::fixture(),
        }
    }
}

impl GetVersionTags for GetVersionTagsStub {
    fn get_version_tags(&self) -> Result<VersionTags, Report<GetVersionTagsError>> {
        Ok(self.tags.clone())
    }
}

impl FromServices for GetVersionTagsStub {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self::default())
    }
}
