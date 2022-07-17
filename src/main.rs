use anyhow::Context;
use regex::Regex;
use reqwest::header::USER_AGENT;
use scraper::{Html, Selector};
use std::env;
use url::Url;

struct Item {
    title: String,
    point: i32,
    price: i32,
}

async fn http_get(url: Url) -> anyhow::Result<String> {
    let product = env!("CARGO_BIN_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(USER_AGENT, format!("{}/{}", product, version))
        .send()
        .await?;
    Ok(response.text().await?)
}

async fn parse(html: String) -> anyhow::Result<Item> {
    let document = Html::parse_document(&html);
    // select title element
    let title_selector = Selector::parse("#collection-title").expect("invalid selector");
    let title = document
        .select(&title_selector)
        .next()
        .map(|e| e.text().fold(String::new(), |a, s| a + s.trim()))
        .context("title not found")?;
    // select price element
    let prices = (0..4)
        .filter_map(|i| {
            let price_selector =
                Selector::parse(format!("#hulk_buy_price_popover_volume_{i}").as_str())
                    .expect("invalid selector");
            document
                .select(&price_selector)
                .next()
                .map(|e| e.text().fold(String::new(), |a, s| a + s.trim()))
                .and_then(|s| {
                    Regex::new("[^0-9]")
                        .expect("invalid regex")
                        .replace_all(&s, "")
                        .parse::<i32>()
                        .ok()
                })
                .map(|price| (i, price))
        })
        .collect::<Vec<(usize, i32)>>();
    let price = prices.last().context("price not found")?.1;
    // select point element
    let points = (0..4)
        .filter_map(|i| {
            let price_selector = Selector::parse(format!("#hulk_buy_points_volume_{i}").as_str())
                .expect("invalid selector");
            document
                .select(&price_selector)
                .next()
                .map(|e| e.text().fold(String::new(), |a, s| a + s.trim()))
                .and_then(|s| {
                    Regex::new("([0-9]+)pt")
                        .expect("invalid regex")
                        .captures(&s)
                        .and_then(|m| m.get(1).unwrap().as_str().parse::<i32>().ok())
                })
                .map(|price| (i, price))
        })
        .collect::<Vec<(usize, i32)>>();
    let point = points.last().context("price not found")?.1;
    Ok(Item {
        title,
        point,
        price,
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url_string = env::args().nth(1).unwrap();
    let url = Url::parse(url_string.as_str())?;
    let response = http_get(url).await?;
    let item = parse(response).await?;
    println!(
        "{} = {} - {} ({}%): {}",
        item.price - item.point,
        item.price,
        item.point,
        (f64::from(item.point) / f64::from(item.price) * 100.0).round(),
        item.title
    );
    Ok(())
}
