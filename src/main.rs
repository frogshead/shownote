use std::process::Command;
use std::{env, fs, io, str::FromStr};

use opml::{self, OPML};
use reqwest;
use rss;
use select::document::Document;
use select::predicate::Name;

#[derive(Debug)]
#[allow(dead_code)]
enum Feed {
    Rss,
    Atom,
}

#[derive(Debug)]
pub struct Podcast {
    name: String,
    feed: Feed,
    url: Option<String>,
    description: Option<String>,
}

fn main() {
    let mut file_name = "src/fav-podcasts.opml".to_string();
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        file_name = args[1].to_string()
    }
    println!("{:?}", args);
    let podcasts = get_podcasts(&file_name);
    let selection = print_podcasts(&podcasts);
    get_episodes(podcasts.iter().nth(selection.into()).unwrap());
}

fn print_podcasts(podcasts: &Vec<Podcast>) -> u8 {
    for (pos, podcast) in podcasts.into_iter().enumerate() {
        println!(
            "[{}] {} - {}",
            pos,
            podcast.name,
            podcast.url.as_ref().unwrap_or(&"default".to_string())
        )
    }
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    println!("Read: {}", buffer);
    let selection: u8 = buffer.trim().parse().unwrap();
    selection
}
fn get_episodes(podcast: &Podcast) {
    println!("Fetching podcast feed {}... ", podcast.name);
    let url = podcast.url.as_ref().unwrap();
    let c = reqwest::blocking::get(url).unwrap().text().unwrap();
    let rss = rss::Channel::from_str(&c).unwrap();

    println!("{} has {} episodes", podcast.name, rss.items.len());
    let content = rss.items.iter().nth(0).unwrap().content.as_ref().unwrap();
    Document::from(content.as_str())
        .find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .for_each(|x| ff(x));
}
fn ff(url: &str) -> () {
    if cfg!(macos) {
        Command::new(r#"open"#).arg(url).spawn().unwrap();
    } else {
        Command::new("cmd")
            .arg("/c")
            .arg("start")
            .arg(url)
            .spawn()
            .expect(url);
    }
}

fn get_podcasts(file_name: &str) -> Vec<Podcast> {
    let xml = fs::read_to_string(file_name).expect("Reading the file failed");
    let opml = OPML::new(&xml).expect("Non Valid OPML/XML file");

    let mut podcasts = vec![];
    for outline in opml.body.outlines {
        let podcast: Podcast = Podcast {
            name: outline.text,
            url: outline.xml_url,
            // feed: match outline.type {
            //     Some(feed_type) => Feed::Rss,
            //     None => Feed::Rss

            // },
            feed: Feed::Rss,
            description: outline.description,
        };
        podcasts.push(podcast);
    }
    println!();
    println!("Total number of podcasts: {}", podcasts.len());
    podcasts
}
