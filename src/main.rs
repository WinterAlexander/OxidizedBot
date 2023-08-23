use std::env;
use html_parser::Dom;
use std::error::Error;
use std::fmt::Write;
use serde_json::Value;

#[derive(Debug)]
struct Committer {
    username: String,
    commit_streak: u32,
    longest_streak: u32,
}

struct YoutubeStats {
    view_count: u32,
    subscriber_count: u32
}

async fn getOnlineMakerKingPlayers() -> Result<u32, Box<dyn Error>> {
    return Ok(reqwest::get(format!("https://makerkinggame.com/api/playersonline"))
        .await?
        .text()
        .await?
        .parse()?);
}

async fn subscribers(channelId: &str) -> Result<YoutubeStats, Box<dyn Error>> {
    let youtubeApiKey = env::var("GOOGLE_API_KEY")?;
    let resp = reqwest::get(format!("https://www.googleapis.com/youtube/v3/channels\
            ?part=statistics&id={channelId}&key={youtubeApiKey}"))
        .await?
        .text()
        .await?;

    let v: Value = serde_json::from_str(&resp)?;

    let stats = v.get("items").ok_or("invalid json")?
        .as_array().ok_or("invalid json")?
        .get(0).ok_or("invalid json")?
        .as_object().ok_or("invalid json")?
        .get("statistics").ok_or("invalid json")?
        .as_object().ok_or("invalid json")?;

    return Ok(YoutubeStats {
        subscriber_count: stats.get("subscriberCount").ok_or("invalid json")?
            .as_str().ok_or("invalid json")?.parse()?,
        view_count: stats.get("viewCount").ok_or("invalid json")?
            .as_str().ok_or("invalid json")?.parse()?
    });
}

async fn commit_streak(username: &str) -> Result<Committer, Box<dyn Error>> {
    let resp = reqwest::get(format!("https://streak-stats.demolab.com/?user={username}"))
        .await?
        .text()
        .await?;

    let dom = Dom::parse(&resp)?;

    let err = "DOM parse failed";
    let base = dom.children[0].element().ok_or(err)?.children[2]
        .element()
        .ok_or(err)?;

    Ok(Committer {
        username: String::from(username),
        commit_streak: extract_dom_node_text(&base.children[3])
            .ok_or(err)?
            .trim()
            .parse()?,
        longest_streak: extract_dom_node_text(&base.children[4])
            .ok_or(err)?
            .trim()
            .parse()?,
    })
}

fn extract_dom_node_text(node: &html_parser::Node) -> Option<&str> {
    Some(
        node.element()?.children[1].element()?.children[0]
            .element()?
            .children[0]
            .text()?
            .trim(),
    )
}

async fn print_commits() -> Result<String, Box<dyn Error>> {
    let users = [
        "WinterAlexander",
        "MartensCedric",
        "RealWilliamWells",
        "Davidster",
    ];
    let mut committers: Vec<Committer> = Vec::new();

    for user in users {
        committers.push(commit_streak(user).await?);
    }

    committers.sort_by(|c1, c2| {
        c2.commit_streak
            .cmp(&c1.commit_streak)
            .then(c2.longest_streak.cmp(&c1.longest_streak))
    });

    let mut s = String::new();

    for (i, comitter) in committers.iter().enumerate() {
        let Committer {
            username,
            commit_streak,
            longest_streak,
        } = comitter;

        writeln!(
            s,
            "#{}: {username}'s commit streak: {commit_streak} (longest: {longest_streak})",
            i + 1,
        )?;
    }

    Ok(s)
}
use teloxide::prelude::*;
use teloxide::types::InputFile;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        if let Some(text) = msg.text() {
            if text.contains("@OxidizeddBot") {
                let lower_text = text.to_lowercase();

                if lower_text.contains("commit streak") {
                    let result = print_commits()
                        .await.map_err(|err| err.to_string());
                    match result {
                        Ok(bot_msg) => {
                            bot.send_message(msg.chat.id, bot_msg).await?;
                        }
                        Err(err) => {
                            log::error!("{err}");
                        }
                    };
                } else if lower_text.contains("a dice") {
                    bot.send_dice(msg.chat.id).await?;
                } else if lower_text.contains("cedric")
                    && lower_text.contains("subscribers") {

                    let result = subscribers("UCAfczIdDUxnzoI_kiB72y8A")
                        .await.map_err(|err| err.to_string());

                    match result {
                        Ok(stats) => {
                            let subs = stats.subscriber_count;
                            let views = stats.view_count;

                            let status = match subs {
                                0..=999 => "yikes!",
                                1000..=1499 => "lame.",
                                1500..=1999 => "good progress.",
                                2000..=9999 => "does he think of himself as some sort of algorithm guru??",
                                10000..=49999 => "wow hello mister relevant, have a nice day.",
                                50000..=99999 => "when is he gonna make a shoutout to MakerKing?",
                                100_000..=199_999 => "you think he can pass thin matrix??",
                                200_000..=499_999 => "this is the stage where its time for him to make a Youtube apology video.",
                                500_000..=999_999 => "is he still really making OpenGL videos???",
                                _ => "congrats on your million subs, deep down you're still a scrub."
                            };

                            bot.send_message(msg.chat.id,
                                             format!("Cedric has {subs} subscribers and {views} channel views, {status}"))
                                .await?;
                        }
                        Err(err) => {
                            log::error!("{err}");
                        }
                    };
                } else if lower_text.contains("makerking")
                    && lower_text.contains("players") {

                    let result = getOnlineMakerKingPlayers()
                        .await.map_err(|err| err.to_string());

                    match result {
                        Ok(count) => {

                            let status = match count {
                                0 => "yikes.",
                                1 => "what a poor lonely loser.",
                                2 => "romantic.",
                                3 => "cool.",
                                4 => "decent.",
                                5 => "nice.",
                                6 => "is it party time?",
                                7 => "crazy",
                                _ => "WOOOO"
                            };

                            bot.send_message(msg.chat.id,
                                             format!("There are {count} players playing \
                                             MakerKing right now, {status}"))
                                .await?;
                        }
                        Err(err) => {
                            log::error!("{err}");
                        }
                    };
                } else {
                    bot.send_video(msg.chat.id, InputFile::file("wtf-marion.mp4"))
                        .await?;
                }
            }
        }

        Ok(())
    })
    .await;
}
