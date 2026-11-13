use std::borrow::Cow;

use anyhow::{Context, anyhow, bail};
use reqwest::{
    Client, IntoUrl, Method, RequestBuilder, Response, StatusCode,
    multipart::{Form, Part},
};
use serde::Deserialize;
use url::Url;

use crate::auth::{oauth2::OAuth2State, oauth2_providers::DeviantArt};

pub struct DeviantArtClient<'a> {
    creds: &'a mut OAuth2State<DeviantArt>,
    reqwest: Client,
}

// See https://deviantart.readme.io/reference/stash_submit
pub struct StashSubmitArgs<'a> {
    pub title: Option<&'a str>,
    pub artist_comments: Option<&'a str>,
    pub tags: Option<&'a [String]>,
    pub original_url: Option<&'a Url>,
    pub is_dirty: Option<bool>,

    // This kind of needed to be 'static instead 'a, due multipart
    // Form from reqwest dont have lifetime
    pub file: Option<(Cow<'static, [u8]>, &'a str)>,
    pub itemid: Option<u64>,
    pub stack: Option<Stack<'a>>,
    pub no_ai: Option<bool>,
    pub is_ai_generated: Option<bool>,
}

pub enum Stack<'a> {
    #[expect(unused)]
    ExistingStack(u64),
    #[expect(unused)]
    NewStack(&'a str),
}

// See https://deviantart.readme.io/reference/stash_publish
pub struct StashPublishArgs<'a> {
    pub mature: MatureLevel<'a>,
    pub feature: Option<bool>,
    pub allow_comments: Option<bool>,
    pub display_resolution: Option<DisplayResolution>,

    pub allow_free_download: Option<bool>,
    pub add_watermark: Option<bool>,
    pub tags: Option<&'a [String]>,
    pub location_tag: Option<&'a str>,
    pub groups: Option<&'a [&'a str]>,
    pub group_folders: Option<&'a [&'a str]>,
    pub is_ai_generated: Option<bool>,
    pub no_ai: Option<bool>,
    pub item_id: u64,
    // Currently galleryids, subject tags and license_options are not implemented
}

pub struct StashPublishResult {
    pub url: Url,
    pub deviation_id: String,
}

#[repr(u8)]
pub enum DisplayResolution {
    Original = 0,
    #[expect(unused)]
    Resolution400px = 1,
    #[expect(unused)]
    Resolution600px = 2,
    #[expect(unused)]
    Resolution800px = 3,
    #[expect(unused)]
    Resolution900px = 4,
    #[expect(unused)]
    Resolution1024px = 5,
    #[expect(unused)]
    Resolution1280px = 6,
    #[expect(unused)]
    Resolution1600px = 7,
    #[expect(unused)]
    Resolution1920px = 8,
}

pub enum MatureLevel<'a> {
    None,
    #[expect(unused)]
    Strict(&'a [MatureClassification]),
    #[expect(unused)]
    Moderate(&'a [MatureClassification]),
}

pub enum MatureClassification {
    #[expect(unused)]
    Nudity,
    #[expect(unused)]
    Sexual,
    #[expect(unused)]
    Gore,
    #[expect(unused)]
    Language,
    #[expect(unused)]
    Ideology,
}

static DA_API_VER: &str = "20240701";

static STASH_SUBMIT_URL: &str = "https://www.deviantart.com/api/v1/oauth2/stash/submit";
static STASH_PUBLISH_URL: &str = "https://www.deviantart.com/api/v1/oauth2/stash/publish";

pub struct StashSubmitResult {
    pub item_id: u64,
    #[expect(unused)]
    pub stack_name: Option<String>,
    #[expect(unused)]
    pub stack_id: Option<u64>,
}

macro_rules! insert_text {
    ($args:ident, $form:ident at $field_name_form:ident <= $field_name:ident) => {
        if let Some(x) = $args.$field_name {
            $form = $form.text(stringify!($field_name_form), format!("{x}"));
        }
    };
}

impl<'a> DeviantArtClient<'a> {
    pub fn new(creds: &'a mut OAuth2State<DeviantArt>) -> Self {
        Self {
            creds,
            reqwest: reqwest::Client::new(),
        }
    }

    async fn create_req<Url: IntoUrl>(
        &mut self,
        method: Method,
        url: Url,
    ) -> anyhow::Result<RequestBuilder> {
        Ok(self
            .reqwest
            .request(method, url)
            .bearer_auth(
                self.creds
                    .get_access_token()
                    .await
                    .context("Cannot get DeviantArt access token")?,
            )
            .header("dA-minor-version", DA_API_VER))
    }

    pub async fn submit_stash(
        &mut self,
        args: StashSubmitArgs<'_>,
    ) -> anyhow::Result<StashSubmitResult> {
        let mut form = Form::new();

        insert_text!(args, form at title <= title);
        insert_text!(args, form at artist_comments <= artist_comments);
        insert_text!(args, form at original_url <= original_url);
        insert_text!(args, form at is_dirty <= is_dirty);
        insert_text!(args, form at itemid <= itemid);
        insert_text!(args, form at noai <= no_ai);
        insert_text!(args, form at is_ai_generated <= is_ai_generated);

        if let Some((file, filename)) = args.file {
            form = form.part("file", Part::bytes(file).file_name(filename.to_string()));
        }

        if let Some(stack) = args.stack {
            assert!(
                args.itemid.is_none(),
                "Stack can only be assigned for new submissions not existing one"
            );

            match stack {
                Stack::ExistingStack(id) => {
                    form = form.text("stackid", format!("{id}"));
                }
                Stack::NewStack(name) => {
                    form = form.text("stack", name.to_string());
                }
            }
        }

        if let Some(tags) = args.tags {
            for tag in tags {
                form = form.text("tags", tag.to_string());
            }
        }

        let response = self
            .create_req(Method::POST, STASH_SUBMIT_URL)
            .await
            .context("Creating request")?
            .multipart(form)
            .send()
            .await
            .context("Sending request")?;

        let status = response.status();
        if status.is_client_error() {
            return Err(Self::process_error(response).await);
        } else if status != StatusCode::OK {
            bail!("Unexpected DeviantArt response status code {status}");
        }

        #[derive(Deserialize)]
        pub struct SuccessResponse {
            status: String,
            itemid: u64,
            stack: Option<String>,
            stackid: Option<u64>,
        }

        let response_bytes = response
            .bytes()
            .await
            .context("Receiving success response bytes")?;
        let response = serde_json::from_slice::<SuccessResponse>(&response_bytes)
            .context("Deserializing success response")?;

        // Its not documented anywhere? expect it be success
        if response.status != "success" {
            bail!(
                "Expected 'success' for status field but DeviantArt responded with {}",
                response.status
            );
        }

        Ok(StashSubmitResult {
            item_id: response.itemid,
            stack_name: response.stack,
            stack_id: response.stackid,
        })
    }

    pub async fn publish_stash(
        &mut self,
        args: StashPublishArgs<'_>,
    ) -> anyhow::Result<StashPublishResult> {
        let mut form = Form::new();

        insert_text!(args, form at feature <= feature);
        insert_text!(args, form at allow_comments <= allow_comments);
        insert_text!(args, form at allow_free_download <= allow_free_download);
        insert_text!(args, form at add_watermark <= add_watermark);
        insert_text!(args, form at location_tag <= location_tag);
        insert_text!(args, form at is_ai_generated <= is_ai_generated);
        insert_text!(args, form at noai <= no_ai);

        form = form.text("itemid", format!("{}", args.item_id));

        if let Some(display_resolution) = args.display_resolution {
            form = form.text(
                "display_resolution",
                format!("{}", display_resolution as u8),
            );
        }

        if let Some(tags) = args.tags {
            form = form.text("tags", tags.join(","));
        }

        if let Some(groups) = args.groups {
            form = form.text("groups", groups.join(","));
        }

        if let Some(group_folders) = args.group_folders {
            form = form.text("group_folders", group_folders.join(","));
        }

        match &args.mature {
            MatureLevel::None => {
                form = form.text("is_mature", "false");
            }
            MatureLevel::Strict(_) => {
                form = form
                    .text("is_mature", "true")
                    .text("mature_level", "strict");
            }
            MatureLevel::Moderate(_) => {
                form = form
                    .text("is_mature", "true")
                    .text("mature_level", "moderate");
            }
        }

        if let MatureLevel::Strict(mature_classifications)
        | MatureLevel::Moderate(mature_classifications) = args.mature
        {
            let mut classifications = Vec::new();
            for item in mature_classifications {
                classifications.push(match item {
                    MatureClassification::Nudity => "nudity",
                    MatureClassification::Sexual => "sexual",
                    MatureClassification::Gore => "gore",
                    MatureClassification::Language => "language",
                    MatureClassification::Ideology => "ideology",
                });
            }
            form = form.text("mature_classification", classifications.join(","));
        }

        let response = self
            .create_req(Method::POST, STASH_PUBLISH_URL)
            .await
            .context("Creating request")?
            .multipart(form)
            .send()
            .await
            .context("Sending request")?;

        let status = response.status();
        if status.is_client_error() {
            return Err(Self::process_error(response).await);
        } else if status != StatusCode::OK {
            bail!("Unexpected DeviantArt response status code {status}");
        }

        #[derive(Deserialize)]
        pub struct SuccessResponse {
            status: String,
            url: Url,
            deviationid: String,
        }

        let response_bytes = response
            .bytes()
            .await
            .context("Receiving success response bytes")?;
        let response = serde_json::from_slice::<SuccessResponse>(&response_bytes)
            .context("Deserializing success response")?;

        // Its not documented anywhere? expect it be success
        if response.status != "success" {
            bail!(
                "Expected 'success' for status field but DeviantArt responded with {}",
                response.status
            );
        }

        Ok(StashPublishResult {
            url: response.url,
            deviation_id: response.deviationid,
        })
    }

    async fn process_error(response: Response) -> anyhow::Error {
        #[derive(Deserialize, Debug)]
        struct ErrorResponse {
            // Expect unused because the error relies on Debug impl
            #[expect(unused)]
            error: String,
            #[expect(unused)]
            error_description: String,
        }

        let response_bytes = match response
            .bytes()
            .await
            .context("Receiving error response bytes")
        {
            Ok(x) => x,
            Err(e) => return e,
        };
        let error = match serde_json::from_slice::<ErrorResponse>(&response_bytes)
            .context("Deserializing error response")
        {
            Ok(x) => x,
            Err(e) => return e,
        };

        anyhow!("Error from server: {error:?}")
    }
}
