use std::{io::{Cursor}, fs};

use dialoguer::MultiSelect;
use error_chain::error_chain;
use home::home_dir;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Content {
  name: String,
  download_url: Option<String>,
}

fn main() -> Result<()> {
  let user_repo_arg = std::env::args().nth(1).or(get_config()).expect("Expected \"{user}/{repo}\" as arg or in \"$HOME/.boilerplate\"");
  let (user, repo) = user_repo_arg.split_once("/").expect("Expected arg to be formatted as \"{user}/{repo}\"");
  let path = std::env::args().nth(2).unwrap_or("".to_string());

  let client = reqwest::blocking::Client::builder()
    .user_agent("boilerfiles")
    .build()?;

  let contents = get_contents(&client, &user.to_string(), &repo.to_string(), &path)?;
  let files: Vec<Content> = contents.into_iter().filter(|content| content.download_url.is_some()).collect();
  let file_names: Vec<String> = files.iter().map(|content| content.name.clone()).collect();
  let selections: Vec<&Content> = MultiSelect::new()
      .items(&file_names)
      .interact()?
      .into_iter()
      .map(|index| files.iter().nth(index))
      .filter(|file| file.is_some())
      .map(|x| x.unwrap())
      .collect();

  for file in selections.into_iter() {
    if std::path::Path::new(&[".", &file.name].join("/")).exists() {
      println!("File already exists, skipping: {}", &file.name);
      continue;
    }
    let download_url = file.download_url.clone().unwrap();
    let res = client.get(download_url).send()?;
    match res.status().is_success() {
      true => {
        let bytes = res.bytes()?;
        let mut file = std::fs::File::create(&file.name)?;
        let mut content =  Cursor::new(bytes);
        std::io::copy(&mut content, &mut file)?;
      },
      false => println!("Download failed: {}", file.name),
    }
  }

  Ok(())
}

fn get_contents(client: &Client, user: &String, repo: &String, path: &String) -> Result<Vec<Content>> {
  let url = ["https://api.github.com/repos", user, repo, "contents", path].join("/");
  let res = client
    .get(url)
    .send()?;
  // TODO: Error handling
  let contents = res.json()?;

  Ok(contents)
}

fn get_config() -> Option<String> {
  let home_dir = home_dir().unwrap();
  let home_path = home_dir.to_str().unwrap();
  let config_path = &[home_path, ".boilerfiles"].join("/");
  let config = fs::read_to_string(config_path);

  return match config {
    Ok(data) => Some(data),
    Err(_error) => None,
  }
}
