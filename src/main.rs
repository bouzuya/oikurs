use regex::Regex;
use reqwest::header::USER_AGENT;
use reqwest::Response;
use scraper::{Html, Selector};
use std::cmp;
use std::env;
use url::Url;

struct Item {
    title: String,
    points: i32,
    price: i32,
}

async fn http_get(url: Url) -> anyhow::Result<Response> {
    let product = "oikurs";
    let version = "0.1.0";
    let client = reqwest::Client::new();
    Ok(client
        .get(url)
        .header(USER_AGENT, format!("{}/{}", product, version))
        .send()
        .await?)
}

async fn parse(response: Response) -> anyhow::Result<Item> {
    let out = response.text().await?;
    let html = out.to_string();
    let document = Html::parse_document(&html);
    // select title element
    let title_selector = Selector::parse(".series-detail-title").unwrap();
    let title = document
        .select(&title_selector)
        .nth(0)
        .map_or(String::new(), |e| {
            e.text().fold(String::new(), |a, s| a + s)
        });
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
    // select discount price element
    let discount_price_selector =
        Selector::parse(".series-price-box-price.discount-price").unwrap();
    let discount_price = document
        .select(&discount_price_selector)
        .nth(0)
        .map(|e| e.text().fold(String::new(), |a, s| a + s))
        .map_or(0, |s| {
            Regex::new("[^0-9]")
                .unwrap()
                .replace_all(&s, "")
                .parse::<i32>()
                .unwrap()
        });
    // select points element
    let amazon_points_selector = Selector::parse(".series-price-box-price.amazon-points").unwrap();
    let amazon_points = document
        .select(&amazon_points_selector)
        .nth(0)
        .map(|e| e.text().fold(String::new(), |a, s| a + s))
        .map_or(0, |s| {
            Regex::new("([0-9]+)ポイント")
                .unwrap()
                .captures(&s)
                .map_or(0, |m| m.get(1).unwrap().as_str().parse::<i32>().unwrap())
        });
    let points = cmp::max(regular_price - discount_price, amazon_points);
    Ok(Item {
        title,
        points,
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
        item.price - item.points,
        item.price,
        item.points,
        (f64::from(item.points) / f64::from(item.price) * 100.0).round(),
        item.title
    );
    Ok(())
}
