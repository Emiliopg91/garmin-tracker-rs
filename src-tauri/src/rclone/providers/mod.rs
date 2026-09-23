use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::rclone::{RCloneClient, RCloneTrait, errors};

pub mod dropbox;
pub mod onedrive;

#[derive(Serialize, Deserialize)]
pub enum CloudProvider {
    OneDrive,
    DropBox,
}

pub struct OneDrive;
pub struct DropBox;

impl CloudProvider {
    pub async fn is_configured(&self) -> errors::Result<bool> {
        match self {
            CloudProvider::OneDrive => RCloneClient::<OneDrive>::default().is_configured().await,
            CloudProvider::DropBox => RCloneClient::<DropBox>::default().is_configured().await,
        }
    }

    pub async fn configure(&self) -> errors::Result<()> {
        match self {
            CloudProvider::OneDrive => RCloneClient::<OneDrive>::default().configure().await,
            CloudProvider::DropBox => RCloneClient::<DropBox>::default().configure().await,
        }
    }

    pub async fn upload<P>(&self, path: P) -> errors::Result<()>
    where
        P: AsRef<Path>,
    {
        match self {
            CloudProvider::OneDrive => RCloneClient::<OneDrive>::default().upload(path).await,
            CloudProvider::DropBox => RCloneClient::<DropBox>::default().upload(path).await,
        }
    }
}
