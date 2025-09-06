use std::{env, io, str::FromStr};

use opml::{self, OPML};
use webbrowser;
use reqwest;
use rss;
use select::document::Document;
use select::predicate::Name;
use ratatui::{
    prelude::*,
    widgets::{List, ListItem, ListState, Block, Borders},
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

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

#[derive(Debug, Clone)]
pub struct Episode {
    title: String,
    description: Option<String>,
    pub_date: Option<String>,
    content: Option<String>,
}

fn main() {
    let mut podcasts_url = "https://raw.githubusercontent.com/frogshead/shownote/main/src/fav-podcasts.opml".to_string();
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        podcasts_url = args[1].to_string()
    }
    println!("{:?}", args);
    let podcasts = get_podcasts(&podcasts_url);
    let podcast_selection = print_podcasts(&podcasts);
    
    if let Some(podcast_index) = podcast_selection {
        let selected_podcast = podcasts.iter().nth(podcast_index).unwrap();
        
        match get_episodes(selected_podcast) {
            Ok(episodes) => {
                let episode_selection = select_episode(&episodes);
                if let Some(episode_index) = episode_selection {
                    if let Some(selected_episode) = episodes.get(episode_index) {
                        open_episode_links(selected_episode);
                    }
                } else {
                    println!("Exiting without opening any links.");
                }
            }
            Err(e) => {
                println!("Error fetching episodes: {}", e);
            }
        }
    } else {
        println!("Exiting without opening any links.");
    }
}

fn print_podcasts(podcasts: &Vec<Podcast>) -> Option<usize> {
    match setup_terminal() {
        Ok(mut terminal) => {
            match run_podcast_selector(&mut terminal, podcasts) {
                Ok(selection) => {
                    if let Err(e) = restore_terminal(&mut terminal) {
                        eprintln!("Failed to restore terminal: {}", e);
                    }
                    selection
                }
                Err(e) => {
                    eprintln!("Failed to run podcast selector: {}", e);
                    if let Err(restore_err) = restore_terminal(&mut terminal) {
                        eprintln!("Failed to restore terminal: {}", restore_err);
                    }
                    fallback_podcast_selection(podcasts)
                }
            }
        }
        Err(_) => {
            println!("Terminal UI not available, falling back to index selection");
            fallback_podcast_selection(podcasts)
        }
    }
}

fn fallback_podcast_selection(podcasts: &Vec<Podcast>) -> Option<usize> {
    for (pos, podcast) in podcasts.iter().enumerate() {
        println!(
            "[{}] {} - {}",
            pos,
            podcast.name,
            podcast.url.as_ref().unwrap_or(&"No URL".to_string())
        );
    }
    println!("Enter the number of the podcast you want to select (or 'q' to quit):");
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    let input = buffer.trim();
    
    if input == "q" || input == "quit" {
        return None;
    }
    
    match input.parse::<usize>() {
        Ok(selection) => Some(selection.min(podcasts.len().saturating_sub(1))),
        Err(_) => Some(0)
    }
}
fn get_episodes(podcast: &Podcast) -> Result<Vec<Episode>, Box<dyn std::error::Error>> {
    println!("Fetching podcast feed {}... ", podcast.name);
    let url = podcast.url.as_ref().unwrap();
    let c = reqwest::blocking::get(url)?.text()?;
    let rss = rss::Channel::from_str(&c)?;

    println!("{} has {} episodes", podcast.name, rss.items.len());
    
    let mut episodes = Vec::new();
    for item in rss.items.iter() {
        let episode = Episode {
            title: item.title().unwrap_or("Untitled Episode").to_string(),
            description: item.description().map(|s| s.to_string()),
            pub_date: item.pub_date().map(|s| s.to_string()),
            content: item.content().map(|s| s.to_string()),
        };
        episodes.push(episode);
    }
    
    Ok(episodes)
}
fn select_episode(episodes: &[Episode]) -> Option<usize> {
    match setup_terminal() {
        Ok(mut terminal) => {
            match run_episode_selector(&mut terminal, episodes) {
                Ok(selection) => {
                    if let Err(e) = restore_terminal(&mut terminal) {
                        eprintln!("Failed to restore terminal: {}", e);
                    }
                    selection
                }
                Err(e) => {
                    eprintln!("Failed to run episode selector: {}", e);
                    if let Err(restore_err) = restore_terminal(&mut terminal) {
                        eprintln!("Failed to restore terminal: {}", restore_err);
                    }
                    fallback_episode_selection(episodes)
                }
            }
        }
        Err(_) => {
            println!("Terminal UI not available, falling back to index selection");
            fallback_episode_selection(episodes)
        }
    }
}

fn fallback_episode_selection(episodes: &[Episode]) -> Option<usize> {
    for (pos, episode) in episodes.iter().enumerate() {
        let date_str = episode.pub_date.as_ref().map(|s| s.as_str()).unwrap_or("No date");
        println!("[{}] {} ({})", pos, episode.title, date_str);
    }
    println!("Enter the number of the episode you want to select (or 'q' to quit):");
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    let input = buffer.trim();
    
    if input == "q" || input == "quit" {
        return None;
    }
    
    match input.parse::<usize>() {
        Ok(selection) => Some(selection.min(episodes.len().saturating_sub(1))),
        Err(_) => Some(0)
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

struct EpisodeApp {
    episodes: Vec<String>,
    state: ListState,
}

impl EpisodeApp {
    fn new(episodes: &[Episode]) -> EpisodeApp {
        let episode_names: Vec<String> = episodes
            .iter()
            .map(|e| {
                let date_str = e.pub_date.as_ref().map(|s| s.as_str()).unwrap_or("No date");
                format!("{} ({})", e.title, date_str)
            })
            .collect();
        
        let mut state = ListState::default();
        if !episode_names.is_empty() {
            state.select(Some(0));
        }
        
        EpisodeApp {
            episodes: episode_names,
            state,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.episodes.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.episodes.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

fn run_episode_selector(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, episodes: &[Episode]) -> Result<Option<usize>, Box<dyn std::error::Error>> {
    let mut app = EpisodeApp::new(episodes);

    loop {
        terminal.draw(|f| episode_ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    KeyCode::Enter => {
                        return Ok(Some(app.state.selected().unwrap_or(0)));
                    }
                    _ => {}
                }
            }
        }
    }
}

fn episode_ui(f: &mut Frame, app: &mut EpisodeApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(f.area());

    let items: Vec<ListItem> = app
        .episodes
        .iter()
        .map(|episode| ListItem::new(episode.as_str()))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Select an Episode (Latest First)"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[0], &mut app.state);

    let help = ratatui::widgets::Paragraph::new("Use ↑/↓ or j/k to navigate, Enter to select, q/Esc to quit")
        .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help, chunks[1]);
}

fn open_episode_links(episode: &Episode) {
    println!("Opening links for episode: {}", episode.title);
    
    if let Some(content) = &episode.content {
        Document::from(content.as_str())
            .find(Name("a"))
            .filter_map(|n| n.attr("href"))
            .for_each(|x| open_urls_to_browser(x));
    } else {
        println!("No content available for this episode");
    }
}

fn run_podcast_selector(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, podcasts: &Vec<Podcast>) -> Result<Option<usize>, Box<dyn std::error::Error>> {
    let mut app = PodcastApp::new(podcasts);

    loop {
        terminal.draw(|f| podcast_ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    KeyCode::Enter => {
                        return Ok(Some(app.state.selected().unwrap_or(0)));
                    }
                    _ => {}
                }
            }
        }
    }
}

struct PodcastApp {
    podcasts: Vec<String>,
    state: ListState,
}

impl PodcastApp {
    fn new(podcasts: &Vec<Podcast>) -> PodcastApp {
        let podcast_names: Vec<String> = podcasts
            .iter()
            .map(|p| format!("{} - {}", p.name, p.url.as_ref().unwrap_or(&"No URL".to_string())))
            .collect();
        
        let mut state = ListState::default();
        if !podcast_names.is_empty() {
            state.select(Some(0));
        }
        
        PodcastApp {
            podcasts: podcast_names,
            state,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.podcasts.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.podcasts.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

fn podcast_ui(f: &mut Frame, app: &mut PodcastApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(f.area());

    let items: Vec<ListItem> = app
        .podcasts
        .iter()
        .map(|podcast| ListItem::new(podcast.as_str()))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Select a Podcast"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[0], &mut app.state);

    let help = ratatui::widgets::Paragraph::new("Use ↑/↓ or j/k to navigate, Enter to select, q/Esc to quit")
        .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help, chunks[1]);
}

fn open_urls_to_browser(url: &str) -> () {
    match webbrowser::open(url){
        Ok(_) => (),
        Err(_) => println!("Cannot open shownote url: {}", url)
    }
}

fn get_podcasts(file_name: &str) -> Vec<Podcast> {
    let xml= reqwest::blocking::get(file_name).expect("Can not fetch the opml from url").text().expect("Cannot convert response to text file");
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
