use anyhow::Context as _;
use serenity::model::application::command::Command;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::prelude::Context;

fn get_ch(
    cmd: &serenity::model::prelude::application_command::ApplicationCommandInteraction,
) -> anyhow::Result<u64> {
    let ch = cmd
        .data
        .options
        .iter()
        .find(|opt| opt.name == "ch")
        .expect("no ch option");

    let ch = ch
        .value
        .as_ref()
        .context("no ch value")?
        .as_str()
        .context("no value")?
        .parse::<u64>()
        .context("parse error")?;
    Ok(ch)
}

impl crate::bot::Bot {
    pub async fn run_command_join(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> anyhow::Result<String> {
        let ch = get_ch(&command)?;
        let gid = command.guild_id.context("not in guild")?;

        self.add_call_state(gid.0, command.channel_id.into())?;

        let man = songbird::get(&ctx).await.expect("init songbird").clone();
        let handler = man.join(gid, ch).await;
        handler.1.context("join failed")?;

        {
            let mut handler = handler.0.lock().await;
            handler.deafen(true).await.context("deafen failed")?;
        }

        Ok("got it!".to_string())
    }

    pub async fn run_command_leave(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> anyhow::Result<String> {
        let man = songbird::get(&ctx).await.expect("init songbird").clone();
        let gid = command.guild_id.context("not in guild")?;
        let cid = command.channel_id;
        let has_handler = man.get(gid).is_some();

        self.erase_call_state(gid.0)?;

        if has_handler {
            man.remove(gid).await.context("leave failed")?;
        } else {
            return Ok("Not in a voice channel".into());
        }

        Ok("bye!".to_string())
    }

    pub async fn register_commands_vc(&self, ctx: &Context) -> anyhow::Result<()> {
        Command::create_global_application_command(&ctx.http, |command| {
            command
                .name("cw-join")
                .description("join")
                .create_option(|option| {
                    use serenity::model::channel::ChannelType;
                    option
                        .name("ch")
                        .description("channel to join")
                        .kind(CommandOptionType::Channel)
                        .channel_types(&[ChannelType::Voice])
                        .required(true)
                })
        })
        .await
        .context("command cw-join registration failed")?;

        Command::create_global_application_command(&ctx.http, |command| {
            command.name("cw-leave").description("leave")
        })
        .await
        .context("command cw-leave registration failed")?;

        Command::create_global_application_command(&ctx.http, |command| {
            command
                .name("cw-play")
                .description("play")
                .create_option(|option| {
                    option
                        .name("str")
                        .description("string")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
        .await
        .context("command cw-play registration failed")?;

        Ok(())
    }
}
