use html_parser::Dom;
use std::error::Error;
use std::fmt::Write;

#[derive(Debug)]
struct Committer {
    username: String,
    commit_streak: u32,
    longest_streak: u32,
}

fn commit_streak(username: &str) -> Result<Committer, Box<dyn Error>> {
    let resp =
        reqwest::blocking::get(format!("https://streak-stats.demolab.com/?user={username}"))?
            .text()?;

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

fn print_commits() -> Result<String, Box<dyn Error>> {
    let users = [
        "WinterAlexander",
        "MartensCedric",
        "RealWilliamWells",
        "Davidster",
    ];
    let mut committers: Vec<Committer> = Vec::new();

    for user in users {
        committers.push(commit_streak(user)?);
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
                if text.to_lowercase().contains("commit streak") {
                    let result = print_commits().map_err(|err| err.to_string());
                    match result {
                        Ok(bot_msg) => {
                            bot.send_message(msg.chat.id, bot_msg).await?;
                        }
                        Err(err) => {
                            log::error!("{err}");
                        }
                    };
                } else if text.to_lowercase().contains("a dice") {
                    bot.send_dice(msg.chat.id).await?;
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
