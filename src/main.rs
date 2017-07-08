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
use std::env;
use std::io::Read;
use std::str::FromStr;
use url::Url;

struct Item {
    title: String,
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
    let title_element = document.select(&title_selector).nth(0).unwrap();
    let title = title_element.text().fold(String::new(), |a, s| a + s);
    // select price element
    let price_selector = Selector::parse("#series-bulkPrice-text").unwrap();
    let price_element = document.select(&price_selector).nth(0).unwrap();
    let price_string = price_element.text().fold(String::new(), |a, s| a + s);
    // price_string -> price
    let re = Regex::from_str("[^0-9]").unwrap();
    let s = re.replace_all(&price_string, "");
    let price = s.parse::<i32>().unwrap();
    Item { title, price }
}

fn main() {
    let url_string = env::args().nth(1).unwrap();
    let headers = build_headers();
    let url = build_url(&url_string);
    let response = http_get(url, headers);
    let item = parse(response);
    println!("{}: {}", item.title, item.price);
}
