

use std::env;
use std::error::Error;
use html_parser::Dom;
use std::fmt::Write;
use tokio::task;

#[derive(Debug)]
struct Committer
{
    username: String,
    commit_streak: u32,
    longest_streak: u32
}

fn commit_streak(username: &str) -> Result<Committer, Box<dyn Error>> {

    let resp = reqwest::blocking::get("https://streak-stats.demolab.com/?user=".to_owned() + username)?.text()?;

    let dom = Dom::parse(&resp)?;

    let err = "DOM parse failed";
    let base = dom.children[0].element().ok_or(err)?
        .children[2].element().ok_or(err)?;

    Ok(Committer {
        username: String::from(username),
        commit_streak: base.children[3].element().ok_or(err)?
            .children[1].element().ok_or(err)?
            .children[0].element().ok_or(err)?
            .children[0].text().ok_or(err)?.trim().parse()?,
        longest_streak: base.children[4].element().ok_or(err)?
            .children[1].element().ok_or(err)?
            .children[0].element().ok_or(err)?
            .children[0].text().ok_or(err)?.trim().parse()?,
    })
}

fn print_commits() -> Result<String, Box<dyn Error>> {
    let users = ["WinterAlexander", "MartensCedric", "RealWilliamWells", "Davidster"];
    let mut committers: Vec<Committer> = Vec::new();

    for user in users {
        committers.push(commit_streak(user)?);
    }

    committers.sort_by(|c1, c2| c2.commit_streak.cmp(&c1.commit_streak)
        .then(c2.longest_streak.cmp(&c1.longest_streak)));

    let mut s = String::new();

    for (i, comitter) in committers.iter().enumerate() {

        writeln!(s,
                 "#{}: {}'s commit streak: {} (longest: {})",
                 i + 1,
                 comitter.username,
                 comitter.commit_streak,
                 comitter.longest_streak)?;
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
            if(text.contains("@OxidizeddBot")) {

                if(text.to_lowercase().contains("commit streak")) {

                    let res: Result<String, String> = task::spawn_blocking(|| {
                        let res: Result<String, Box<dyn Error>> = print_commits();
                        match res {
                            Ok(v) => {
                                return Ok(v);
                            }
                            Err(shit) => {
                                return Err(shit.to_string());
                            }
                        }
                    }).await.unwrap();

                    if(res.is_ok())
                    {
                        bot.send_message(msg.chat.id, res.unwrap()).await?;
                    }

                }
                else if(text.to_lowercase().contains("a dice")) {
                    bot.send_dice(msg.chat.id).await?;
                } else {
                    bot.send_video(msg.chat.id, InputFile::file("wtf-marion.mp4")).await?;
                }
            }
        }

        Ok(())
    })
    .await;
}
