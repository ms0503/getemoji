use lazy_regex::Lazy;
use lazy_regex::Regex;
use lazy_regex::lazy_regex;
use reqwest::Url;
use serde::Deserialize;
use serde::Serialize;
use std::env;
use std::error::Error;
use std::process;
use tokio::fs;
use tokio::task::JoinHandle;

const MISSKEY_DOWNLOAD_HOST: &str = "media.misskeyusercontent.com";
const MISSKEY_HOST: &str = "misskey.io";

static MISSKEY_DOWNLOAD_EMOJI_PATH_PATTERN: Lazy<Regex> = lazy_regex!(
    r#"^/misskey/webpublic-[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89a-d][0-9a-f]{3}-[0-9a-f]{12}\.[\w.\-_~]+$"#
);
static MISSKEY_GET_EMOJI_PATH_PATTERN: Lazy<Regex> = lazy_regex!(r#"^/emojis/[0-9_a-z]{2,}$"#);

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("must be specified emojis' name");
        process::exit(1);
    }
    let emojis = {
        let args = args.to_owned();
        let emojis = &args[1..];
        emojis.to_owned()
    };
    let mut pool: Vec<JoinHandle<_>> = vec![];
    for emoji in emojis {
        pool.push(tokio::spawn(async move {
            match download_emoji(&emoji).await {
                Err(err) => eprintln!(":{}: : {}", emoji, err),
                Ok(_) => println!(":{}: : Success", emoji)
            }
        }));
    }
    for handle in pool {
        let _ = handle.await;
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct EmojiData {
    pub icon: IconData
}

#[derive(Debug, Deserialize, Serialize)]
struct IconData {
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub url: String
}

fn get_suffix(mime_type: &str) -> &str {
    match mime_type {
        "image/avif" => "avif",
        "image/gif" => "gif",
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        _ => unreachable!()
    }
}

async fn get_emoji_data(name: &str) -> Result<EmojiData, Box<dyn Error>> {
    let url = validate_emoji_url(name)?;
    let res = reqwest::get(url).await?.json::<EmojiData>().await?;
    Ok(res)
}

async fn download_emoji(name: &str) -> Result<(), Box<dyn Error>> {
    let emoji = get_emoji_data(name).await?;
    let url = validate_download_url(&emoji.icon.url)?;
    let data = reqwest::get(url).await?.bytes().await?;
    match fs::write(
        format!("{}.{}", name, get_suffix(&emoji.icon.media_type)),
        data
    )
    .await
    {
        Err(_) => Err("Failed to write image file".into()),
        Ok(_) => Ok(())
    }
}

fn validate_emoji_url(name: &str) -> Result<Url, Box<dyn Error>> {
    let url = Url::parse(&format!("https://{}/emojis/{}", MISSKEY_HOST, name))?;
    let is_valid = url.scheme() == "https"
        && url.host_str() == Some(MISSKEY_HOST)
        && url.username().is_empty()
        && url.password().is_none()
        && (url.port().is_none() || url.port() == Some(443))
        && url.query().is_none()
        && url.fragment().is_none()
        && MISSKEY_GET_EMOJI_PATH_PATTERN.is_match(url.path());
    if is_valid {
        Ok(url)
    } else {
        Err("Invalid Misskey emoji URL".into())
    }
}

fn validate_download_url(raw: &str) -> Result<Url, Box<dyn Error>> {
    let url = Url::parse(raw)?;
    let is_valid = url.scheme() == "https"
        && url.host_str() == Some(MISSKEY_DOWNLOAD_HOST)
        && url.username().is_empty()
        && url.password().is_none()
        && (url.port().is_none() || url.port() == Some(443))
        && url.query().is_none()
        && url.fragment().is_none()
        && MISSKEY_DOWNLOAD_EMOJI_PATH_PATTERN.is_match(url.path());
    if is_valid {
        let mut canonical_url = Url::parse(&format!("https://{}", MISSKEY_DOWNLOAD_HOST))?;
        canonical_url.set_path(url.path());
        Ok(canonical_url)
    } else {
        Err("Invalid Misskey emoji media URL".into())
    }
}
