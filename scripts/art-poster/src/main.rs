use std::{env::args_os, path::Path};

use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

use crate::{
    auth::{oauth2::OAuth2State, oauth2_providers::DeviantArt},
    interface::Parameters,
};

mod api;
mod auth;
mod interface;
mod ui;
mod uploaders;

#[derive(Deserialize, Serialize)]
struct Data {
    da_account: OAuth2State<DeviantArt>,
}

static DATA_PATH: &str = "./.art-post-data.json";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if args_os().len() >= 2 {
        // argv[0] might not exists so length 0 and length 1
        // is valid
        bail!("Program is not expecting any argument");
    }

    let mut data;
    if Path::new(DATA_PATH).exists() {
        let raw_data = fs::read_to_string(&DATA_PATH)
            .await
            .with_context(|| format!("Loading art post data at {DATA_PATH}"))?;
        data = serde_json::from_str::<Data>(&raw_data).context("Deserializing art post data")?;
    } else {
        data = Data {
            da_account: OAuth2State::<DeviantArt>::new_interactive()
                .await
                .context("Cannot initialize oauth2 state for DeviantArt")?,
        };
    }

    // Trigger things needed to get ready use access token
    data.da_account
        .get_access_token()
        .await
        .context("Cannot get acess token for this app to access DeviantArt")?;

    // The accounts is logged in lets say the data
    save_data(&data).await.context("Saving data")?;

    let params = Parameters::ask()
        .await
        .context("Cannot ask for art upload parameters")?;

    println!("Uploading");
    if let Err(e) = uploaders::upload(&mut data, &params)
        .await
        .context("Cannot upload")
    {
        // Don't really care about error, try save in best effort
        let _ = save_data(&data).await;
        return Err(e).context("Cannot upload");
    }
    save_data(&data).await.context("Saving data")?;
    Ok(())
}

async fn save_data(data: &Data) -> anyhow::Result<()> {
    let mut file = File::create(DATA_PATH).await.context("Openign data file")?;
    let serialized = serde_json::to_string_pretty(data).context("Serializing data")?;

    file.write_all(serialized.as_bytes())
        .await
        .context("Cannot write serialized data")?;
    Ok(())
}
