use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use walkdir::WalkDir;
use std::env;
use chrono::Local;


// function to find the best alternative to a 404 based on URL and all the files in the docs folder
fn find_best_alternative(url: &str) -> Option<String> {
    fn break_url_into_words(url: &str) -> String {
        let re = Regex::new(r"^https:\/\/|hasura\.io|\.[a-z]+|\/docs\/|latest/").unwrap();
        let result = re.replace_all(url, "");
        let index = result.find("/").unwrap();
        let result = &result[index+1..];
        let result = result.replace("-", " ");


        // TODO: We'll need to deal with anchor tags at some point, too...
        

        return result.trim_end_matches("/").to_string();

    }

    // loop over every file in the docs folder and find the one with the most matching words to the break_url_into_words function
    let mut best_match = 0;
    let mut best_match_file = String::new();
    for entry in WalkDir::new("../../graphql-engine-mono/docs/docs/") {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let mut file = File::open(path).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let words = break_url_into_words(url);
            let mut matches = 0;
            for word in words.split(" ") {
                if contents.contains(word) {
                    matches += 1;
                }
            }
            if matches > best_match {
                best_match = matches;
                best_match_file = path.to_str().unwrap().to_string();
            }
        }
    }

    if best_match > 0 {
        return Some(best_match_file);
    } else {
        return None;
    }
}

fn generate_nginx_config(old_path: &str, new_path: &str) -> String {
    // modify old_path to remove everything before and including the /docs/docs/ part
    let re = Regex::new(r"^https:\/\/|hasura\.io|\.[a-z]+|\/docs\/|latest/").unwrap();
    let old_path = re.replace_all(old_path, "");
    let old_path = old_path.trim_end_matches("/");
    let old_path = old_path.trim_start_matches("/");
    let old_path = old_path.trim_start_matches("docs/");
    let old_path = old_path.trim_start_matches("docs");

    let re = Regex::new(r"/docs/docs/(.+)\\.mdx").unwrap();
    let new_path = re.replace_all(new_path, "");
    let new_path = new_path.trim_start_matches("../../graphql-engine-mono/docs/docs/");
    let new_path = new_path.replace(".mdx", "/");
    let new_path = new_path.trim_end_matches("/");

    

    let nginx_config = r#"
    # TEST ME: https://hasura.io/docs/latest/{{new_path}}/
    location = /docs/latest/{{old_path}} {
        return 301 https://$host/docs/latest/{{new_path}}/;
    }
    "#;

    let config = nginx_config
        .replace("{{old_path}}", old_path)
        .replace("{{new_path}}", new_path);

    config.to_string()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let urls = &args[1..];
    let today = Local::now().format("%m/%d/%Y").to_string();

    let date_header = r#"
    ##################################################################
    # DOCS Redirect ({{today}})
    ##################################################################
    "#;

    print!("{}", date_header.replace("{{today}}", &today));
    
    for url in urls.iter() {
        if let Some(alternative) = find_best_alternative(url) {
            let nginx_config = generate_nginx_config(url, &alternative);
            println!("{}", nginx_config);
        } else {
            println!("No alternative found for {}", url);
        }
    }
}