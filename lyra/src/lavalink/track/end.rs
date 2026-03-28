use lavalink_rs::{
    client::LavalinkClient,
    model::{
        events::TrackEnd,
        track::{TrackData, TrackLoadData, TrackLoadType},
    },
};
use rand::seq::SliceRandom;
use twilight_model::id::Id;

use crate::{
    command::require::cleanup_now_playing_message_and_play,
    error::lavalink::ProcessResult,
    lavalink::{CorrectTrackInfo, PlaylistAwareTrackData, UnwrappedData},
};

#[tracing::instrument(err, skip_all, name = "track_end")]
pub(super) async fn impl_end(
    lavalink: LavalinkClient,
    _session_id: String,
    event: &TrackEnd,
) -> ProcessResult {
    let guild_id = event.guild_id;
    tracing::debug!(
        "guild {} ended   {:?}",
        guild_id.0,
        event.track.info.checked_title()
    );

    let Some(player) = lavalink.get_player_context(guild_id) else {
        tracing::debug!(?guild_id, "track ended via forced disconnection");

        return Ok(());
    };
    let data = player.data_unwrapped();

    let advancing_disabled = data.read().await.queue().advancing_disabled().await;
    if advancing_disabled {
        tracing::debug!(?guild_id, "track ended forcefully");
    } else {
        tracing::debug!(?guild_id, "track ended normally");
        let mut data_w = data.write().await;

        let cdata = &*lavalink.data_unwrapped();
        data_w.cleanup_now_playing_message(cdata).await;

        let queue = data_w.queue_mut();

        // Capture autoplay state and last requester before advancing
        let autoplay_enabled = queue.autoplay();
        let last_requester = queue.current().map(|item| item.requester());

        queue.advance();
        if let Some(index) = queue.current_index() {
            cleanup_now_playing_message_and_play(&player, cdata, index, &mut data_w).await?;
        } else if autoplay_enabled {
            // Queue is exhausted but autoplay is on — load a related track
            drop(data_w);

            let requester =
                last_requester.unwrap_or_else(|| Id::new(1));

            if let Some(track_data) =
                load_autoplay_track(&lavalink, guild_id, &event.track).await
            {
                let mut data_w = data.write().await;
                data_w.queue_mut().enqueue(
                    vec![PlaylistAwareTrackData::from(track_data)],
                    requester,
                );
                if let Some(index) = data_w.queue().current_index() {
                    let cdata = &*lavalink.data_unwrapped();
                    player.play_now(data_w.queue()[index].data()).await?;
                    drop(data_w);
                }
            }
        }
        // (if not autoplay and queue ended, just do nothing — playback stops)
    }

    Ok(())
}

/// Build a query to find related tracks.
///
/// For YouTube tracks, uses YouTube Mix (Radio) for better recommendations.
/// For all other sources, falls back to a YouTube search with the track's metadata.
fn build_autoplay_query(track: &TrackData) -> String {
    let info = &track.info;

    // Try YouTube Mix for YouTube content
    if let Some(ref uri) = info.uri {
        if let Some(video_id) = extract_youtube_video_id(uri) {
            return format!(
                "https://www.youtube.com/watch?v={}&list=RD{}",
                video_id, video_id
            );
        }
    }

    // Fallback: search YouTube with the track's artist and title
    let title = &info.title;
    let author = &info.author;
    format!("ytsearch:{author} {title}")
}

/// Extract a YouTube video ID from a URL.
fn extract_youtube_video_id(uri: &str) -> Option<&str> {
    if uri.contains("youtube.com/watch") || uri.contains("music.youtube.com/watch") {
        uri.split("v=")
            .nth(1)?
            .split(&['&', '#', '?'][..])
            .next()
    } else if uri.contains("youtu.be/") {
        uri.split("youtu.be/")
            .nth(1)?
            .split(&['?', '#', '&'][..])
            .next()
    } else {
        None
    }
}

/// Load a single related track for autoplay.
async fn load_autoplay_track(
    lavalink: &LavalinkClient,
    guild_id: lavalink_rs::model::GuildId,
    finished_track: &TrackData,
) -> Option<TrackData> {
    let query = build_autoplay_query(finished_track);
    tracing::debug!(?guild_id, %query, "autoplay: loading related tracks");

    let loaded = match lavalink.load_tracks(guild_id, &query).await {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!(?guild_id, "autoplay: failed to load tracks: {:?}", e);
            return None;
        }
    };

    match loaded.load_type {
        TrackLoadType::Playlist => {
            // YouTube Mix returns a playlist — pick a random track that isn't the same
            let Some(TrackLoadData::Playlist(mut playlist)) = loaded.data else {
                return None;
            };
            let finished_uri = finished_track.info.uri.as_deref();

            // Shuffle for variety
            let mut rng = rand::rng();
            playlist.tracks.shuffle(&mut rng);

            playlist
                .tracks
                .into_iter()
                .find(|t| t.info.uri.as_deref() != finished_uri)
        }
        TrackLoadType::Search => {
            // ytsearch: results — pick the first that isn't the same
            let Some(TrackLoadData::Search(tracks)) = loaded.data else {
                return None;
            };
            let finished_uri = finished_track.info.uri.as_deref();

            tracks
                .into_iter()
                .find(|t| t.info.uri.as_deref() != finished_uri)
        }
        TrackLoadType::Track => {
            // Direct track match
            if let Some(TrackLoadData::Track(track)) = loaded.data {
                Some(track)
            } else {
                None
            }
        }
        TrackLoadType::Empty | TrackLoadType::Error => {
            tracing::debug!(?guild_id, "autoplay: no results for query");
            None
        }
    }
}
