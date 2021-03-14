use std::{fs, str::FromStr};

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
    let file_name = "src/fav-podcasts.opml";
    let podcasts = get_podcasts(&file_name);
    get_episodes(podcasts.iter().nth(17).unwrap());
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
        .for_each(|x| println!("{}", x));
}
fn get_podcasts(file_name: &str) -> Vec<Podcast> {
    let xml = fs::read_to_string(file_name).expect("Reading the file failed");
    let opml = OPML::new(&xml).expect("Non Valid OPML/XML file");
    println!("Version: {:?}", opml.version);

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
        println!("[]{} - {:?}", podcast.name, podcast.url);
        podcasts.push(podcast);
    }
    println!();
    println!("Total number of podcasts: {}", podcasts.len());
    podcasts
}
