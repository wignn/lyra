use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::{
    command::{
        model::{BotGuildSlashCommand, GuildSlashCmdCtx},
        require,
    },
    core::model::response::initial::message::create::RespondWithMessage,
    error::CommandResult,
};

/// Toggles autoplay. When enabled, related tracks will be automatically played when the queue ends.
#[derive(CommandModel, CreateCommand)]
#[command(name = "autoplay", contexts = "guild")]
pub struct Autoplay;

impl BotGuildSlashCommand for Autoplay {
    async fn run(self, mut ctx: GuildSlashCmdCtx) -> CommandResult {
        let player = require::player(&ctx)?;
        let data = player.data();
        let mut data_w = data.write().await;
        let enabled = data_w.queue_mut().toggle_autoplay();
        drop(data_w);

        let message = if enabled {
            "🔄 Enabled autoplay. Related tracks will be automatically played when the queue ends."
        } else {
            "⏹️ Disabled autoplay."
        };
        ctx.out(message).await?;
        Ok(())
    }
}
