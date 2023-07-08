use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

pub fn run(options: &[CommandDataOption]) -> String {
    let count = options
        .iter()
        .find(|option| option.name == "count")
        .map(|option| option.value.as_ref().unwrap().as_u64().unwrap())
        .unwrap_or(1);

    let count = count.clamp(1, 32);

    "にゃーん".to_string().repeat(count as usize)
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("neko")
        .description("猫のように鳴く")
        .create_option(|option| {
            option
                .name("count")
                .description("にゃーんの回数")
                .kind(CommandOptionType::Integer)
                .min_int_value(1)
                .max_int_value(32)
                .required(false)
        })
}
