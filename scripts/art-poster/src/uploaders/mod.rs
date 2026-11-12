use anyhow::Context;

use crate::interface::Parameters;

mod deviantart;

pub async fn upload(data: &mut crate::Data, params: &Parameters) -> anyhow::Result<()> {
    let metadata = serde_json::to_string(&params.metadata).context("Cannot serialize metadta")?;

    println!("Uploading to DeviantArt");
    deviantart::upload(data, &metadata, params)
        .await
        .context("Uploading to DeviantArt")?;
    println!("Uploaded to everything");
    Ok(())
}
