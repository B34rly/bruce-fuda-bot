use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use anyhow::Context as _;
use chrono::{FixedOffset, Timelike, Utc};
use poise::serenity_prelude::{self as serenity};
use shuttle_poise::ShuttlePoise;
use shuttle_secrets::SecretStore;
use std::time::Duration;

use rand::seq::SliceRandom;

struct Data {
    morningAnnouncements: Mutex<Vec<String>>,
    curfewAnnouncements: Mutex<Vec<String>>,
    is_loop_running: AtomicBool,
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/* /// Responds with "world!"
#[poise::command(slash_command)]
async fn hello(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("world!").await?;

    Ok(())
} */

/// Add a new message into my announcements
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS"
)]
async fn add_message(
    ctx: Context<'_>,
    #[description = "At which time should I send this message? [Either \"morning\" (8:45am) or \"curfew\" (11pm)]"]
    time_of_day: std::string::String,
    #[description = "What message should I tell the students of NCSS?"]
    new_message: std::string::String,
) -> Result<(), Error> {
    match time_of_day.as_ref() {
        "morning" => {
            let _announcement = {
                let mut morning_announcements = ctx.data().morningAnnouncements.lock().unwrap();
                morning_announcements.push(new_message.clone());
            };
            ctx.say(format!(
                "Ok, got it! I'll say \"{}\" during my {} announcements.",
                &new_message, time_of_day
            ))
            .await?
        }
        "curfew" => {
            let _announcement = {
                let mut curfew_announcements = ctx.data().curfewAnnouncements.lock().unwrap();
                curfew_announcements.push(new_message.clone());
            };
            ctx.say(format!(
                "Ok, got it! I'll say \"{}\" during my {} announcements.",
                &new_message, time_of_day
            ))
            .await?
        }
        _ => {
            ctx.say(
                "I don't understand what time you want me to say that, speak properly next time.",
            )
            .await?
        }
    };

    {
        let curfew_announcements =
            serde_json::to_string(&*ctx.data().curfewAnnouncements.lock().unwrap().clone())
                .unwrap();
        let morning_announcements =
            serde_json::to_string(&*ctx.data().morningAnnouncements.lock().unwrap().clone())
                .unwrap();

        println!("{}", morning_announcements);
        println!("{}", curfew_announcements);

        println!(
            "New Command Added! \n    Morning Announcemetns: {:?}\n    Curfew Announcements {:?}",
            morning_announcements, curfew_announcements
        );
        match fs::write("morningAnnouncements.json", &morning_announcements) {
            Ok(n) => println!("morning announcements successfully saved"),
            Err(e) => println!("Erorr saving morning announcements, {}", e),
        }
        match fs::write("curfewAnnouncements.json", &curfew_announcements) {
            Ok(n) => println!("curfew announcements successfully saved"),
            Err(e) => println!("Erorr saving curfew announcements, {}", e),
        }
    }

    Ok(())
}

/// Remove an announcement using it's index. Use list_announcements to get the index of messages.
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS"
)]
async fn remove_message(
    ctx: Context<'_>,
    #[description = "From which message group should I remove the message? [Either \"morning\" (8:45am) or \"curfew\" (11pm)]"]
    time_of_day: std::string::String,
    #[description = "What's the index of the message to remove?"]
    message_index_str: std::string::String,
) -> Result<(), Error> {
    let message_index: usize;
    match message_index_str.parse::<usize>() {
        Ok(_n) => message_index = message_index_str.parse::<usize>().unwrap(),
        Err(_e) => {
            ctx.say("You did not give me an integer!").await?;
            return Ok(());
        }
    };

    match time_of_day.as_ref() {
        "morning" => {
            let response: String;
            let _announcement = {
                let mut morning_announcements: std::sync::MutexGuard<'_, Vec<String>> =
                    ctx.data().morningAnnouncements.lock().unwrap();
                let max_length: usize = morning_announcements.len();
                if message_index >= 0 && message_index < max_length {
                    let removed_msg = morning_announcements.remove(message_index);
                    response = format!(
                        "Index {} of {} announcements, \"{}\" removed.",
                        message_index, time_of_day, removed_msg
                    );
                } else {
                    response = format!(
                        "Index {} out of bounds! Must be between 0 and {}",
                        message_index, max_length
                    );
                }
            };
            ctx.say(response).await?
        }
        "curfew" => {
            let response: String;
            let _announcement = {
                let mut curfew_announcement: std::sync::MutexGuard<'_, Vec<String>> =
                    ctx.data().curfewAnnouncements.lock().unwrap();
                let max_length: usize = curfew_announcement.len();

                if message_index >= 0 && message_index < curfew_announcement.len() {
                    let removed_msg = curfew_announcement.remove(message_index);
                    response = format!(
                        "Index {} of {} announcements, \"{}\" removed.",
                        message_index, time_of_day, removed_msg
                    );
                } else {
                    response = format!(
                        "Index {} out of bounds! Must be between 0 and {}",
                        message_index, max_length
                    );
                }
            };
            ctx.say(response).await?
        }
        _ => {
            ctx.say(
                "I don't understand what time you want me to say that, speak properly next time.",
            )
            .await?
        }
    };

    {
        let curfew_announcements =
            serde_json::to_string(&*ctx.data().curfewAnnouncements.lock().unwrap().clone())
                .unwrap();
        let morning_announcements =
            serde_json::to_string(&*ctx.data().morningAnnouncements.lock().unwrap().clone())
                .unwrap();

        println!("{}", morning_announcements);
        println!("{}", curfew_announcements);

        println!(
            "Command Removed! \n    Morning Announcemetns: {:?}\n    Curfew Announcements {:?}",
            morning_announcements, curfew_announcements
        );
        match fs::write("morningAnnouncements.json", &morning_announcements) {
            Ok(n) => println!("morning announcements successfully saved"),
            Err(e) => println!("Erorr saving morning announcements, {}", e),
        }
        match fs::write("curfewAnnouncements.json", &curfew_announcements) {
            Ok(n) => println!("curfew announcements successfully saved"),
            Err(e) => println!("Erorr saving curfew announcements, {}", e),
        }
    }

    Ok(())
}

/// Say a morning announcement
#[poise::command(slash_command)]
async fn make_a_morning_announcement(ctx: Context<'_>) -> Result<(), Error> {
    let morning_announcements: Vec<String> =
        ctx.data().morningAnnouncements.lock().unwrap().to_vec();
    let response: String;
    if morning_announcements.len() == 0 {
        response = format!("https://cdn.discordapp.com/attachments/1196650681717772298/1196793913294471183/lv_7250253436737080581_20240116232908.mp4?ex=65b8ec30&is=65a67730&hm=e45b3bc4b2851002e18c56c8ece3d779f44448fbd623601cf898c9132ed150dc&");
    } else {
        response = format!(
            "{}",
            morning_announcements
                .choose(&mut rand::thread_rng())
                .as_ref()
                .unwrap()
        );
    };
    ctx.say(response).await?;

    Ok(())
}

/// Say a curfew announcement
#[poise::command(slash_command)]
async fn make_a_curfew_announcement(ctx: Context<'_>) -> Result<(), Error> {
    let curfew_announcements: Vec<String> = ctx.data().curfewAnnouncements.lock().unwrap().to_vec();
    let response: String;
    if curfew_announcements.len() == 0 {
        response = format!("https://cdn.discordapp.com/attachments/1196650681717772298/1196793913294471183/lv_7250253436737080581_20240116232908.mp4?ex=65b8ec30&is=65a67730&hm=e45b3bc4b2851002e18c56c8ece3d779f44448fbd623601cf898c9132ed150dc&");
    } else {
        response = format!(
            "{}",
            curfew_announcements
                .choose(&mut rand::thread_rng())
                .as_ref()
                .unwrap()
        );
    };
    ctx.say(response).await?;

    Ok(())
}

/// Output all available messages
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS"
)]
async fn list_announcements(
    ctx: Context<'_>,
    #[description = "Let me know which messages to output [Either \"morning\" or \"curfew\" or neither]"]
    time_of_day: std::string::String,
) -> Result<(), Error> {
    match time_of_day.as_ref() {
        "morning" => {
            let morning_announcements: Vec<String> =
                ctx.data().morningAnnouncements.lock().unwrap().to_vec();
            let mut response: String = String::from("Morning Announcements:");
            if morning_announcements.len() == 0 {
                response = format!("There are no morning announcements added for me to say");
            } else {
                for (index, element) in morning_announcements.into_iter().enumerate() {
                    response += format!("\n    Message at index {} is {}", index, element).as_ref();
                }
                //response = format!("{:?}", morning_announcements);
            };
            ctx.say(response).await?;
        }
        "curfew" => {
            let curfew_announcements: Vec<String> =
                ctx.data().curfewAnnouncements.lock().unwrap().to_vec();
            let mut response: String = String::from("Curfew Announcements:");
            if curfew_announcements.len() == 0 {
                response = format!("There are no curfew announcements added for me to say");
            } else {
                for (index, element) in curfew_announcements.into_iter().enumerate() {
                    response += format!("\n    Message at index {} is {}", index, element).as_ref();
                }
            };
            ctx.say(response).await?;
        }
        _ => {
            let morning_announcements: Vec<String> =
                ctx.data().morningAnnouncements.lock().unwrap().to_vec();
            let curfew_announcements: Vec<String> =
                ctx.data().curfewAnnouncements.lock().unwrap().to_vec();
            let mut response: String = String::from("Morning Announcements: ");
            for (index, element) in morning_announcements.into_iter().enumerate() {
                response += format!("\n    Message at index {} is {}", index, element).as_ref();
            }
            response += "\nCurfew Announcements:".as_ref();
            for (index, element) in curfew_announcements.into_iter().enumerate() {
                response += format!("\n    Message at index {} is {}", index, element).as_ref();
            }
            ctx.say(response).await?;
        }
    };
    Ok(())
}

//create command to run main loop and another to close it
//check if an atomic bool is true first
//then, during the running loop, every 55 seconds check time and if it's 11 or 8:45 run a morning/curfew command

/// Begins the daily announcement loop
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS"
)]
async fn begin_announcements(ctx: Context<'_>) -> Result<(), Error> {
    if !ctx.data().is_loop_running.load(Ordering::Relaxed) {
        tokio::spawn(async move {
            loop {
                let now = Utc::now().with_timezone(&FixedOffset::east_opt(1 * 3600 * 11).unwrap());

                if now.hour() == 8 && now.minute() == 45 {
                    make_a_morning_announcement();
                } else if now.hour() == 23 && now.minute() == 0 {
                    make_a_curfew_announcement();
                }
                //if time...
                println!(
                    "{}",
                    format!(
                        "now {}, now.hour {}, now.minute {}",
                        now.to_string(),
                        now.hour().to_string(),
                        now.minute().to_string()
                    )
                );

                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        });
        ctx.data().is_loop_running.swap(true, Ordering::Relaxed);
        ctx.say("loop should've started!").await?;
    } else {
        ctx.say("loop already running!").await?;
    }
    Ok(())
}

#[shuttle_runtime::main]
async fn poise(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> ShuttlePoise<Data, Error> {
    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                add_message(),
                make_a_morning_announcement(),
                make_a_curfew_announcement(),
                remove_message(),
                begin_announcements(),
                list_announcements(),
            ],
            ..Default::default()
        })
        .token(discord_token)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    morningAnnouncements: Mutex::new(match fs::read_to_string("morningAnnouncements.json") {
                        Ok(_n) => {
                            serde_json::from_str(&fs::read_to_string("morningAnnouncements.json").unwrap())
                                .unwrap()
                        },
                        Err(_) => Vec::new(),
                    }),
                    curfewAnnouncements: Mutex::new(match fs::read_to_string("curfewAnnouncements.json") {
                        Ok(_n) => {
                            serde_json::from_str(&fs::read_to_string("curfewAnnouncements.json").unwrap())
                                .unwrap()
                        },
                        Err(_) => Vec::new(),
                    }),
                    is_loop_running: AtomicBool::new(false),
                })
            })
        })
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(framework.into())
}
