extern crate hyper;
extern crate hyper_native_tls;
extern crate scraper;
extern crate url;
extern crate regex;

use hyper::Client;
use hyper::client::Response;
use hyper::header::{Headers, UserAgent};
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use regex::Regex;
use scraper::{Html, Selector};
use std::cmp;
use std::env;
use std::io::Read;
use url::Url;

struct Item {
    title: String,
    points: i32,
    price: i32,
}

fn build_headers() -> Headers {
    let mut headers = Headers::new();

    let product = "oikurs";
    let version = "0.1.0";
    headers.set(UserAgent(format!("{}/{}", product, version)));

    headers
}

fn build_url(url_string: &str) -> Url {
    Url::parse(url_string).unwrap()
}

fn http_get(url: Url, headers: Headers) -> Response {
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);
    client.get(url).headers(headers).send().unwrap()
}

fn parse(mut response: Response) -> Item {
    let mut out = String::new();
    response.read_to_string(&mut out).unwrap();
    let html = out.to_string();
    let document = Html::parse_document(&html);
    // select title element
    let title_selector = Selector::parse(".series-detail-title").unwrap();
    let title = document
        .select(&title_selector)
        .nth(0)
        .map_or(String::new(),
                |e| e.text().fold(String::new(), |a, s| a + s));
    // select regular price element
    let regular_price_selector = Selector::parse(".series-price-box-price.regular-price").unwrap();
    let regular_price = document
        .select(&regular_price_selector)
        .nth(0)
        .map(|e| e.text().fold(String::new(), |a, s| a + s))
        .map_or(0, |s| {
            Regex::new("[^0-9]")
                .unwrap()
                .replace_all(&s, "")
                .parse::<i32>()
                .unwrap()
        });
    // select price element
    let bulk_price_selector = Selector::parse("#series-bulkPrice-text").unwrap();
    let bulk_price = document
        .select(&bulk_price_selector)
        .nth(0)
        .map(|e| e.text().fold(String::new(), |a, s| a + s))
        .map_or(0, |s| {
            Regex::new("[^0-9]")
                .unwrap()
                .replace_all(&s, "")
                .parse::<i32>()
                .unwrap()
        });
    let price = cmp::max(regular_price, bulk_price);
    // select points element
    let points_selector = Selector::parse(".series-price-box-price.amazon-points").unwrap();
    let points = document
        .select(&points_selector)
        .nth(0)
        .map(|e| e.text().fold(String::new(), |a, s| a + s))
        .map_or(0, |s| {
            Regex::new("([0-9]+)ポイント")
                .unwrap()
                .captures(&s)
                .map_or(0, |m| m.get(1).unwrap().as_str().parse::<i32>().unwrap())
        });
    Item {
        title,
        points,
        price,
    }
}

fn main() {
    let url_string = env::args().nth(1).unwrap();
    let headers = build_headers();
    let url = build_url(&url_string);
    let response = http_get(url, headers);
    let item = parse(response);
    println!("{} = {} - {} ({}%): {}",
             item.price - item.points,
             item.price,
             item.points,
             (f64::from(item.points) / f64::from(item.price) * 100.0).round(),
             item.title);
}
