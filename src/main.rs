use regex::Regex;
use std::env;
use std::fs;
use std::error::Error;
use std::process::Command;
use chrono::Local;

async fn query_algolia(parsed_query: &str) -> Result<String, Box<dyn Error>> {
    // Define nginx config templates
    let nginx_config_latest_template = r"
# TEST ME: https://stage.hasura.io/docs/latest/{{old_path}}
location = /docs/latest/{{old_path}} {
    return 301 https://$host/{{new_path}};
}
";

    let nginx_config_3_0_template = r"
# TEST ME: https://stage.hasura.io/docs/3.0/{{old_path}}
location = /docs/3.0/{{old_path}} {
    return 301 https://$host/docs/3.0/<REPLACE_WITH_NEW_PATH>;
}
";

    // Check if the path contains /docs/3.0 and skip Algolia
    if parsed_query.contains("/docs/3.0") {
        let nginx_config = nginx_config_3_0_template
            .replace("{{old_path}}", &parsed_query["/docs/3.0/".len()..]);
        return Ok(nginx_config);
    }

    // Continue with Algolia processing for /docs/latest
    // Regex to remove https://hasura.io/docs/latest/ from parsed_query
    let re = Regex::new(r#"/docs/latest/"#).unwrap();
    let parsed_query_clean = re.replace_all(parsed_query, "");

    // Then, we'll take the parsed_query and break it into a nice urlEncoded format
    let sent_parsed_query = url_encode(&parsed_query_clean);

    // Make the actual request to Algolia
    let resp = make_algolia_request(&sent_parsed_query).await?;

    // From body, get hits[0].url which is the meat of the json we want
    let url = extract_url_from_algolia_response(&resp)?;

    // Create nginx config for /docs/latest
    let nginx_config = nginx_config_latest_template
        .replace("{{old_path}}", &parsed_query_clean)
        .replace("{{new_path}}", &url);

    Ok(nginx_config)
}

// Helper function to URL encode the parsed query
fn url_encode(parsed_query: &str) -> String {
    parsed_query.replace("/", "%20")
        .replace("-", "%20")
        .replace("#", "%20")
}

// Helper function to make a request to Algolia
async fn make_algolia_request(query: &str) -> Result<String, Box<dyn Error>> {
    let algolia_query = format!("{{ \"params\": \"query={}\" }}", query);
    let resp = reqwest::Client::new()
        .post("https://NS6GBGYACO-dsn.algolia.net/1/indexes/hasura-graphql/query")
        .header("X-Algolia-API-Key", "8f0f11e3241b59574c5dd32af09acdc8")
        .header("X-Algolia-Application-Id", "NS6GBGYACO")
        .body(algolia_query)
        .send().await?;

    let body = resp.text().await?;
    Ok(body)
}

// Helper function to extract URL from Algolia response
fn extract_url_from_algolia_response(response: &str) -> Result<String, Box<dyn Error>> {
    let re = Regex::new(r#""url":"([^"]*)""#).unwrap();
    let url_match = re.captures(response)
        .ok_or("URL not found in Algolia response")?
        .get(1)
        .ok_or("URL capture group not found")?
        .as_str();

    // Strip the url of https://hasura.io/ because the redirect doesn't want it
    let url = url_match.replace("https://hasura.io/", "");

    // If url has an anchor tag in it, remove the final / to make sure we render the correct part of the page
    let url = if url.ends_with('/') {
        url[..url.len() - 1].to_string()
    } else {
        url
    };

    Ok(url)
}

#[tokio::main]

async fn main() -> Result<(), std::io::Error> {
    // command line args
    let args: Vec<String> = std::env::args().collect();
    // today's date in MM-DD-YYYY format
    let today = Local::now().format("%m/%d/%Y").to_string();
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
                println!("✅ -> {}", arg);
                let _ = &config.push_str(&url);
            },
            Err(e) => println!("Error: {}", e),
        }
    }

    let path = "../../hasura.io/redirects/paths";
    env::set_current_dir(path).unwrap();
    let nginx_config = fs::read_to_string("docs.conf").unwrap();

    // get master
    let _output = Command::new("git")
        .arg("checkout")
        .arg("master")
        .output()
        .expect("failed to execute process");

    // let's pull
    let _output = Command::new("git")
        .arg("pull")
        .output()
        .expect("failed to execute process");

    // then make a branch
    let _output = Command::new("git")
        .arg("checkout")
        .arg("-b")
        .arg(format!("rob/redirects/docs-{}", &today))
        .output()
        .expect("failed to execute process");

    // delete the local branches of release-stage and release-prod
    let _output = Command::new("git")
        .arg("branch")
        .arg("-D")
        .arg("release-stage")
        .output()
        .expect("failed to execute process");

    let _output = Command::new("git")
        .arg("branch")
        .arg("-D")
        .arg("release-prod")
        .output()
        .expect("failed to execute process");


    // add config to the bottom of the file
    let _ = fs::write("docs.conf", format!("{}\n{}", nginx_config, config)).expect("Unable to write file");


    // open the file in code
    let _output = Command::new("code")
        .arg("-n")
        .arg("../")
        .output()
        .expect("failed to execute process");

    Ok(())

}
