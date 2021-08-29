use std::{env, io, str::FromStr};

use opml::{self, OPML};
use webbrowser;
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
    let mut podcasts_url = "https://raw.githubusercontent.com/frogshead/shownote/main/src/fav-podcasts.opml".to_string();
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        podcasts_url = args[1].to_string()
    }
    println!("{:?}", args);
    let podcasts = get_podcasts(&podcasts_url);
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
        .for_each(|x| open_urls_to_browser(x));
}
fn open_urls_to_browser(url: &str) -> () {
    match webbrowser::open(url){
        Ok(_) => (),
        Err(_) => println!("Cannot open shownote url: {}", url)
    }
}

fn get_podcasts(file_name: &str) -> Vec<Podcast> {
    let xml: String = reqwest::blocking::get(file_name).expect("Can not fetch the opml from url").text().unwrap();
    let opml = OPML::from_str(&xml).expect("Non Valid OPML/XML file");
    let mut podcasts = vec![];
    for outline in opml.body.outlines {
        let podcast: Podcast = Podcast {
            name: outline.text,
            url: outline.xml_url,
            feed: Feed::Rss,
            description: outline.description,
        };
        podcasts.push(podcast);
    }
    println!();
    println!("Total number of podcasts: {}", podcasts.len());
    podcasts
}
