use serenity::model::application::command::Command;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::prelude::Context;

fn get_ch(
    cmd: &serenity::model::prelude::application_command::ApplicationCommandInteraction,
) -> Result<u64, String> {
    let ch = cmd
        .data
        .options
        .iter()
        .find(|opt| opt.name == "ch")
        .expect("no ch option");

    let ch = ch
        .value
        .as_ref()
        .ok_or("no ch value")?
        .as_str()
        .ok_or("no value")?
        .parse::<u64>()
        .map_err(|_| "parse error")?;
    Ok(ch)
}

impl crate::bot::Bot {
    pub async fn run_command_join(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<String, String> {
        let ch = get_ch(&command)?;
        let gid = command.guild_id.ok_or_else(|| "not in guild".to_string())?;

        self.add_call_state(gid.0, command.channel_id.into())?;

        let man = songbird::get(&ctx).await.expect("init songbird").clone();
        let handler = man.join(gid, ch).await;
        handler.1.map_err(|e| {
            log::error!("join failed: {:?}", e);
            "error occured"
        })?;
        {
            let mut handler = handler.0.lock().await;
            handler.deafen(true).await.map_err(|e| {
                log::error!("deafen failed: {:?}", e);
                "error occured"
            })?;
        }

        Ok("got it!".to_string())
    }

    pub async fn run_command_leave(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<String, String> {
        let man = songbird::get(&ctx).await.expect("init songbird").clone();
        let gid = command.guild_id.ok_or_else(|| "not in guild".to_string())?;
        let cid = command.channel_id;
        let has_handler = man.get(gid).is_some();

        self.erase_call_state(gid.0)?;

        if has_handler {
            if let Err(e) = man.remove(gid).await {
                cid.say(&ctx.http, format!("Failed: {:?}", e))
                    .await
                    .map_err(|e| {
                        log::error!("say error: {:?}", e);
                        "error occured".to_string()
                    })?;
            }

            cid.say(&ctx.http, "Left voice channel")
                .await
                .map_err(|e| {
                    log::error!("say error: {:?}", e);
                    "error occured".to_string()
                })?;
        } else {
            cid.say(&ctx.http, "Not in a voice channel")
                .await
                .map_err(|e| {
                    log::error!("say error: {:?}", e);
                    "error occured".to_string()
                })?;
        }

        Ok("bye!".to_string())
    }

    pub async fn register_commands_vc(&self, ctx: &Context) -> Result<(), ()> {
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
        .map_err(|e| log::error!("error: {:?}", e))?;

        Command::create_global_application_command(&ctx.http, |command| {
            command.name("cw-leave").description("leave")
        })
        .await
        .map_err(|e| log::error!("error: {:?}", e))?;

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
        .map_err(|e| log::error!("error: {:?}", e))?;

        Ok(())
    }
}
