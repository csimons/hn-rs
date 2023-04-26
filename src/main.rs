use std::env;
use std::fs::File;
use std::io::Write;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

#[derive(Debug)]
struct Article {
    title: String,
    link: String,
}

fn parse_articles(xml_blob: &str) -> Vec<Article> {
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

fn main() {
    let rss_data = reqwest::blocking::get("https://news.ycombinator.com/rss")
        .expect("unable to fetch HN site")
        .text()
        .expect("unable to read HN RSS data");

    let articles: Vec<Article> = parse_articles(&rss_data);

    let path = format!("{}/.hn2", env::var("HOME").unwrap());
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
