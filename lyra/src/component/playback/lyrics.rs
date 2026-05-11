use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::{
    command::{
        model::{BotGuildSlashCommand, GuildSlashCmdCtx},
        require,
    },
    core::model::response::initial::message::create::RespondWithMessage,
    error::CommandResult,
    lavalink::lava_lyrics,
};

/// Gets the lyrics of the current track.
#[derive(CreateCommand, CommandModel)]
#[command(name = "lyrics", contexts = "guild")]
pub struct Lyrics;

impl BotGuildSlashCommand for Lyrics {
    async fn run(self, mut ctx: GuildSlashCmdCtx) -> CommandResult {
        let player = require::player(&ctx)?;
        let data = player.data();
        let data_r = data.read().await;

        let track = require::current_track(data_r.queue())?;
        let encoded_track = track.track.data().encoded.clone();
        let track_title = track.track.data().info.title.clone();
        drop(data_r);

        let req_client = reqwest::Client::new();
        let lyrics_result = lava_lyrics::get_lyrics(&req_client, &encoded_track).await;

        match lyrics_result {
            Ok(Some(lyrics)) => {
                let mut embed_description = String::new();
                if let Some(text) = lyrics.text {
                    embed_description = text;
                } else if let Some(lines) = lyrics.lines {
                    embed_description = lines
                        .into_iter()
                        .map(|l| l.line)
                        .collect::<Vec<String>>()
                        .join("\n");
                }

                if embed_description.is_empty() {
                    ctx.out("Lyrics are empty.").await?;
                    return Ok(());
                }

                if embed_description.len() > 4096 {
                    embed_description.truncate(4093);
                    embed_description.push_str("...");
                }

                let embed = twilight_util::builder::embed::EmbedBuilder::new()
                    .title(format!("Lyrics - {}", track_title))
                    .description(embed_description)
                    .color(0x5865F2)
                    .build();

                ctx.respond().embeds(vec![embed]).await?;
            }
            Ok(None) => {
                ctx.out("Lyrics not found.").await?;
            }
            Err(e) => {
                tracing::error!("Failed to fetch lyrics: {}", e);
                ctx.erro("Failed to fetch lyrics.").await?;
            }
        }

        Ok(())
    }
}
