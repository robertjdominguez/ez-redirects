use regex::Regex;
// use reqwest::Url;
use std::error::Error;
// use chrono::Local;


async fn query_algolia(parsed_query: &str) -> Result<(), Box<dyn Error>> {
    // take parsed_query and brake it into words with %20 in between them
    let parsed_query = parsed_query.replace("/", "%20")
    .replace("-", "%20")
    .replace("#", "%20");

    // query and request
    let query: String = format!("{{ \"params\": \"query={}\" }}", parsed_query);
    println!("{}", query);
    let resp = reqwest::Client::new()
        .post("<https://NS6GBGYACO-dsn.algolia.net/1/indexes/hasura-graphql/query>")
        .header("X-Algolia-API-Key", "8f0f11e3241b59574c5dd32af09acdc8")
        .header("X-Algolia-Application-Id", "NS6GBGYACO")
        .body(query);
    let resp = resp.send().await?;
    let body = resp.text().await?;

    // from body, get hits[0].url
    let re = Regex::new(r#""url":"([^"]*)""#).unwrap();
    let url = re.captures(&body).unwrap().get(1).unwrap().as_str();
    println!("{}", url);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // command line args
    let args: Vec<String> = std::env::args().collect();
    // today's date in MM-DD-YYYY format
    let today = Local::now().format("%m/%d/%Y").to_string();

    let date_header = r#"
    ##################################################################
    # DOCS Redirect ({{today}})
    ##################################################################
    "#;


    for arg in args.iter().skip(1) {
        let _ = query_algolia(arg).await;
    }

    // graceful
    Ok(())
}
