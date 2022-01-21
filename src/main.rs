use clap::{IntoApp, StructOpt};
use regex::Regex;
use reqwest::Url;
use scraper::{ElementRef, Selector};
#[macro_use]
extern crate clap;

// a program for crawling all links from html page
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The URL to scrape
    url: String,
    /// regular expression to filter result
    #[clap(short, long)]
    regex: Option<String>,
}

fn main() {
    let args = parse_args();
    let urls = get_links(&args.url)
        .into_iter()
        .filter(|s| args.regex.is_match(s));

    for url in urls {
        println!("{}", url);
    }
}

fn parse_args() -> ParsedArgs {
    let args = Args::parse();

    let regex = get_regex(args.regex);

    let url = Url::parse(&args.url).unwrap_or_else(|e| {
        let mut app = Args::into_app();
        app.error(clap::ErrorKind::InvalidValue, format!("invalid url: {e}"))
            .exit();
    });

    ParsedArgs { url, regex }
}

struct ParsedArgs {
    url: Url,
    regex: Regex,
}

fn get_regex(reg: Option<String>) -> Regex {
    match reg {
        Some(reg) => Regex::new(&reg).unwrap_or_else(|e| {
            Args::into_app()
                .error(clap::ErrorKind::InvalidValue, format!("invalid regex: {e}"))
                .exit()
        }),
        None => Regex::new("").unwrap(),
    }
}

fn get_links(url: &Url) -> impl Iterator<Item = String> {
    let result = request(url);

    let document = scraper::Html::parse_document(&result);

    let links_selector = Selector::parse("a").unwrap();

    fn get_href(element: ElementRef) -> Option<String> {
        element.value().attr("href").map(|f| f.to_string())
    }

    document
        .select(&links_selector)
        .filter_map(get_href)
        .collect::<Vec<_>>()
        .into_iter()
}

fn request(url: &Url) -> String {
    reqwest::blocking::get(url.as_str())
        .unwrap_or_else(|e| {
            Args::into_app()
                .error(clap::ErrorKind::Io, format!("{}", e))
                .exit();
        })
        .text()
        .unwrap_or_else(|e| {
            Args::into_app()
                .error(clap::ErrorKind::Io, format!("{}", e))
                .exit();
        })
}
