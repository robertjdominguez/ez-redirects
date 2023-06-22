use regex::Regex;
use std::env;
use std::fs;
use std::error::Error;
use std::process::Command;
use chrono::Local;

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

    // strip the url of https://hasura.io/ because the redirect doesn't want it
    let re = Regex::new(r#"https://hasura.io/"#).unwrap();
    let url = re.replace_all(url, "");

    // if url has an anchor tag in it, remove the final / to make sure we render the correct part of the page
    let re = Regex::new(r#"\/$"#).unwrap();
    let url = re.replace_all(&url, "");

    
let nginx_config = r"
# TEST ME: https://stage.hasura.io/docs/latest/{{old_path}}
location = /docs/latest/{{old_path}} {
    return 301 https://$$host/{{new_path}};
}
";

    let config = nginx_config
        .replace("{{old_path}}", &parsed_query.to_string())
        .replace("{{new_path}}", &url.to_string());

    Ok(config.to_string())


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

    let path = "../../hasura.io/redirects";
    env::set_current_dir(path).unwrap();
    let nginx_config = fs::read_to_string("redirects.conf").unwrap();

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
        .arg(format!("rob/docs/docs-redirects-{}", &today))
        .output()
        .expect("failed to execute process");

    // add two blank lines to the config variable and re-include our target string
    let final_config = format!("{}\n\n##################################################################\n\nlocation ~ ^/docs/latest/(.*)\\.html$ {{", &config);

    // find this string in the nginx_config variable: location ~ ^/docs/latest/(.*)\.html$ {
    // and insert config string in its place
    let re = Regex::new(r#"#+\s+location ~ \^/docs/latest/\(\.\*\)\\\.html\$ \{"#).unwrap();
    let redir_config = re.replace(&nginx_config, &final_config);
    // convert nginx_config to a string
    let redir_config = redir_config.to_string();

    
    // open redirects.conf and write nginx_config to it
    let _ = fs::write("redirects.conf", redir_config).expect("Unable to write file");

    // open the file in code
    let _output = Command::new("code")
        .arg("-n")
        .arg("../")
        .output()
        .expect("failed to execute process");

    Ok(())

}
