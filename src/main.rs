use regex::Regex;
use std::error::Error;
use chrono::Local;

async fn query_algolia(parsed_query: &str) -> Result<String, Box<dyn Error>> {
    // regex to remove https://hasura.io/docs/latest/ from parsed_query
    let re = Regex::new(r#"https://hasura.io/docs/latest/"#).unwrap();
    let parsed_query = re.replace_all(parsed_query, "");

    // take parsed_query and brake it into words with %20 in between them
    let parsed_query = parsed_query.replace("/", "%20")
    .replace("-", "%20")
    .replace("#", "%20");

    // query and request
    let query: String = format!("{{ \"params\": \"query={}\" }}", parsed_query);
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

    Ok(url.to_string())


}

#[tokio::main]

async fn main() {
    // command line args
    let args: Vec<String> = std::env::args().collect();
    // today's date in MM-DD-YYYY format
    let today = Local::now().format("%m/%d/%Y").to_string();

    let date_header = r#"
    ##################################################################
    # DOCS Redirects ({{today}})
    ##################################################################
    "#;


    println!("{}", date_header.replace("{{today}}", &today));

    for arg in args.iter().skip(1) {
        match query_algolia(arg).await {
            Ok(url) => println!("{}", url),
            Err(e) => println!("Error: {}", e),
        }
    }
}
