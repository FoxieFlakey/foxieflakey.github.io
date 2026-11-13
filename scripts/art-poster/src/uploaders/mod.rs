use anyhow::Context;

use crate::interface::Parameters;
use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};

mod deviantart;
mod my_website;

pub async fn upload(data: &mut crate::Data, params: &Parameters) -> anyhow::Result<()> {
    let metadata = BASE64_STANDARD_NO_PAD
        .encode(serde_json::to_string(&params.metadata).context("Cannot serialize metadta")?);

    println!("Uploading to my website");
    my_website::upload(data, &metadata, params)
        .await
        .context("Uploading to my website")?;
    println!("Uploading to DeviantArt");
    deviantart::upload(data, &metadata, params)
        .await
        .context("Uploading to DeviantArt")?;
    println!("Uploaded to everything");
    Ok(())
}
