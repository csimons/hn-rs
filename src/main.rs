use std::env;
use std::fs::File;
use std::io;
use std::io::{BufRead, Write};
use std::process::Command;
use std::str::FromStr;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

const DOTFILE: &str = ".hn2";

#[derive(Debug)]
struct Article {
    title: String,
    link: String,
}

fn parse_articles_from_xml(xml_blob: &str) -> Vec<Article> {
    let mut reader = Reader::from_str(xml_blob);

    let mut in_title = false;
    let mut in_comments = false;
    let mut current_title = String::new();

    let mut articles: Vec<Article> = Vec::new();

    loop {
        match reader.read_event() {
            Err(e) => panic!("error at position {}: {:?}", reader.buffer_position(), e),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"title" => in_title = true,
                b"comments" => in_comments = true,
                _ => (),
            },
            Ok(Event::End(e)) => match e.name().as_ref() {
                b"title" => in_title = false,
                b"comments" => in_comments = false,
                _ => (),
            },
            Ok(Event::Text(e)) => {
                if in_title {
                    current_title = e.unescape().unwrap().to_string();
                }

                if in_comments {
                    articles.push(Article {
                        title: current_title.clone(), // needed?
                        link: e.unescape().unwrap().to_string(),
                    });
                }
            }
            _ => (),
        }
    }
    articles
}

fn parse_articles_from_disk() -> Vec<Article> {
    let mut articles: Vec<Article> = Vec::new();

    let path = format!("{}/{}", env::var("HOME").unwrap(), DOTFILE);
    let file = File::open(path).unwrap();
    for line_maybe in io::BufReader::new(file).lines() {
        match line_maybe {
            Ok(line) => {
                let tokens = line.split("\t").collect::<Vec<_>>();
                articles.push(Article {
                    title: String::from(tokens[0]),
                    link: String::from(tokens[1]),
                });
            }
            Err(e) => panic!("{}", e),
        };
    }

    articles
}

fn fetch_and_print() {
    let rss_data = reqwest::blocking::get("https://news.ycombinator.com/rss")
        .expect("unable to fetch HN site")
        .text()
        .expect("unable to read HN RSS data");

    let articles: Vec<Article> = parse_articles_from_xml(&rss_data);

    let path = format!("{}/{}", env::var("HOME").unwrap(), DOTFILE);
    let mut f = File::options()
        // .create_new(true)
        .create(true)
        .write(true)
        .open(path)
        .unwrap();

    let mut count = 0;
    for article in articles {
        count += 1;

        println!("{}\t{}", count, article.title);
        writeln!(&mut f, "{}\t{}", article.title, article.link).unwrap();
    }
}

fn open_url(url: &str) {
    let open_cmd = match env::consts::OS {
        "linux" => "xdg-open",
        "macos" => "open",
        "windows" => "start",
        _ => {
            panic!("error: unsupported OS");
        }
    };

    Command::new(open_cmd).args([url]).output().unwrap();
}

fn main() {
    let args = env::args();

    if args.len() == 1 {
        fetch_and_print();
        std::process::exit(0);
    }

    let articles = parse_articles_from_disk();

    for item_str in args.skip(1) {
        let item_num = u64::from_str(&item_str).unwrap();

        let mut current: u64 = 0;
        for article in &articles {
            current += 1;
            if current == item_num {
                open_url(&article.link);
            }
        }
    }
}
