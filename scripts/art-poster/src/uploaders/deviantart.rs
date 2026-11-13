use std::borrow::Cow;

use anyhow::Context;

use crate::{
    api::deviantart::{
        DeviantArtClient, DisplayResolution, MatureLevel, StashPublishArgs, StashSubmitArgs,
    },
    interface::Parameters,
};

pub async fn upload(
    data: &mut crate::Data,
    // JSON encoded which then base64 encoded content of params.metadata
    metadata: &str,
    params: &Parameters,
) -> anyhow::Result<()> {
    let mut api = DeviantArtClient::new(&mut data.da_account);
    let item_id = api
        .submit_stash(StashSubmitArgs {
            title: Some(&params.title),
            artist_comments: Some(&format!(
                "{}\nMetadata (base64):\n{metadata}",
                params.description
            )),
            tags: Some(&params.keywords),
            original_url: Some(&params.metadata.authoritative_url),
            is_dirty: None,
            file: Some((Cow::Owned(params.file.clone()), &params.filename)),
            itemid: None,
            stack: None,
            no_ai: Some(true),
            is_ai_generated: Some(false),
        })
        .await
        .context("Cannot upload to sta.sh")?
        .item_id;

    let result = api
        .publish_stash(StashPublishArgs {
            mature: MatureLevel::None,
            feature: Some(false),
            allow_comments: Some(true),
            display_resolution: Some(DisplayResolution::Original),
            allow_free_download: Some(true),
            add_watermark: Some(false),
            tags: Some(&params.keywords),
            location_tag: None,
            groups: None,
            group_folders: None,
            is_ai_generated: Some(false),
            no_ai: Some(true),
            item_id,
        })
        .await
        .context("Cannot publish uploaded art on DeviantArt")?;

    println!(
        "Uploaded to DeviantArt at {} with Deviation ID of {}",
        result.url, result.deviation_id
    );
    Ok(())
}
