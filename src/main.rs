use regex::Regex;
// use reqwest::Url;
use std::error::Error;
// use chrono::Local;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // command line args
    let args: Vec<String> = std::env::args().collect();
    let parsed_query = &args[1];

    // take the parsed query and keep only everything after https://hasura.io/docs/latest/
    let re = Regex::new(r#"https://hasura.io/docs/latest/(.*)"#).unwrap();
    let parsed_query = re.captures(parsed_query).unwrap().get(1).unwrap().as_str();

    // take parsed_query and brake it into words with %20 in between them
    let parsed_query = parsed_query.replace("/", " ");
    let parsed_query = parsed_query.replace("-", " ");

    println!("{}", parsed_query);
    
    // query and request
    let query = format!("{{\"params\":\"query=\\\"{{\\\"query\\\":\\\"{{\\\"query\\\":\\\"{}\\\"}}\\\"}}\\\"\"}}", parsed_query);
    let resp = reqwest::Client::new()
        .post("https://NS6GBGYACO-dsn.algolia.net/1/indexes/hasura-graphql/query")
        .header("X-Algolia-API-Key", "8f0f11e3241b59574c5dd32af09acdc8")
        .header("X-Algolia-Application-Id", "NS6GBGYACO")
        .body(query);
    let resp = resp.send().await?;
    let body = resp.text().await?;
    
    // from body, get hits[0].url
    let re = Regex::new(r#""url":"([^"]*)""#).unwrap();
    let url = re.captures(&body).unwrap().get(1).unwrap().as_str();
    println!("{}", url);

    // graceful
    Ok(())
}