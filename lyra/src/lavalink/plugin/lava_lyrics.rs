use reqwest::Client;
use serde::Deserialize;
use std::env;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Lyrics {
    pub provider: String,
    pub text: Option<String>,
    pub lines: Option<Vec<LyricLine>>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LyricLine {
    pub line: String,
    pub range: Option<LyricRange>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LyricRange {
    pub start: u64,
    pub end: u64,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum LyricsResponse {
    Lyrics(Lyrics),
    Error {
        timestamp: u64,
        status: u16,
        error: String,
        message: String,
        path: String,
    },
}

pub async fn get_lyrics(
    client: &Client,
    encoded_track: &str,
) -> Result<Option<Lyrics>, reqwest::Error> {
    let host = env::var("SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1".into());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "2333".into());
    let password =
        env::var("LAVALINK_SERVER_PASSWORD").unwrap_or_else(|_| "youshallnotpass".into());
    let is_ssl = env::var("LAVALINK_IS_SSL").unwrap_or_else(|_| "false".into()) == "true";
    let protocol = if is_ssl { "https" } else { "http" };

    let url = format!("{protocol}://{host}:{port}/v4/loadlyrics?encodedTrack={encoded_track}");

    let res = client
        .get(&url)
        .header("Authorization", password)
        .send()
        .await?;

    if res.status() == 404 {
        return Ok(None);
    }

    let body = res.json::<LyricsResponse>().await?;
    match body {
        LyricsResponse::Lyrics(l) => Ok(Some(l)),
        LyricsResponse::Error { .. } => Ok(None),
    }
}
