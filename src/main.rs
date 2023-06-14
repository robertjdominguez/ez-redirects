use regex::Regex;
use std::error::Error;
use chrono::Local;
use arboard::Clipboard;

async fn query_algolia(parsed_query: &str) -> Result<String, Box<dyn Error>> {
    // we'll use some regex to remove https://hasura.io/docs/latest/ from parsed_query
    let re = Regex::new(r#"/docs/latest/"#).unwrap();
    let parsed_query = re.replace_all(parsed_query, "");

    // then, we'll take the parsed_query and break it into a nice urlEncoded format
    let sent_parsed_query = parsed_query.replace("/", "%20")
    .replace("-", "%20")
    .replace("#", "%20");

    // we'll make the actual request
    let query: String = format!("{{ \"params\": \"query={}\" }}", sent_parsed_query);
    let resp = reqwest::Client::new()
        .post("https://NS6GBGYACO-dsn.algolia.net/1/indexes/hasura-graphql/query")
        .header("X-Algolia-API-Key", "8f0f11e3241b59574c5dd32af09acdc8")
        .header("X-Algolia-Application-Id", "NS6GBGYACO")
        .body(query);
    let resp = resp.send().await?;
    let body = resp.text().await?;

    // from body, get hits[0].url which is the meat of the json we want
    let re = Regex::new(r#""url":"([^"]*)""#).unwrap();
    let url = re.captures(&body).unwrap().get(1).unwrap().as_str();

    // strip the url of https://hasura.io/ becaus the redirect doesn't want it
    let re = Regex::new(r#"https://hasura.io/"#).unwrap();
    let url = re.replace_all(url, "");

    // if url has an anchor tag in it, remove the final / to make sure we render the correct part of the page
    let re = Regex::new(r#"\/$"#).unwrap();
    let url = re.replace_all(&url, "");

    
let nginx_config = r#"
# TEST ME: https://hasura.io/{{new_path}}
location = /docs/latest/{{old_path}} {
    return 301 https://$host/{{new_path}};
}
"#;

    let config = nginx_config
        .replace("{{old_path}}", &parsed_query.to_string())
        .replace("{{new_path}}", &url.to_string());

    Ok(config.to_string())


}

#[tokio::main]

async fn main() {
    // command line args
    let args: Vec<String> = std::env::args().collect();
    // today's date in MM-DD-YYYY format
    let today = Local::now().format("%m/%d/%Y").to_string();
    // we'll "init" the clipboard
    let mut clipboard = Clipboard::new().unwrap();
    // we'll create an empty string to hold the final config and redirects
    let mut config = String::new();

let date_header = r#"
##################################################################
# DOCS Redirects ({{today}})
##################################################################
"#;

    // put that sucker at the top
    config.push_str(&date_header.replace("{{today}}", &today));


    println!("{}", date_header.replace("{{today}}", &today));

    for arg in args.iter().skip(1) {
        match query_algolia(arg).await {
            Ok(url) => {
                println!("{}", url);
                config.push_str(&url);
            },
            Err(e) => println!("Error: {}", e),
        }
    }

    // put it on the clipboard to fulfill my laziness
    clipboard.set_text(config).unwrap();
}
