use std::{io::Cursor, fs, time::Instant};

use dialoguer::MultiSelect;
use error_chain::error_chain;
use home::home_dir;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};

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

  let files = fetch_files(&client, &user.to_string(), &repo.to_string(), &path)?;
  let file_names: Vec<String> = files.iter().map(|content| content.name.clone()).collect();
  let selections: Vec<&Content> = MultiSelect::new()
      .items(&file_names)
      .interact()?
      .into_iter()
      .map(|index| files.iter().nth(index))
      .filter(|file| file.is_some())
      .map(|x| x.unwrap())
      .collect();

  if selections.is_empty() {
    println!("{}", "No files selected");
    std::process::exit(0);
  }

  let start = Instant::now();
  for remote_file in selections.into_iter() {
    download_file(remote_file, &client)?;
  }
  println!("\nDone after {}ms", start.elapsed().as_millis());

  return Ok(());
}

fn download_file(remote_file: &Content, client: &Client) -> Result<()> {
  let start = Instant::now();
  let mut sp = Spinner::new(
      Spinners::Dots,
      format!("{}: Downloading", remote_file.name).into(),
  );
  if std::path::Path::new(&[".", &remote_file.name].join("/")).exists() {
    sp.stop_and_persist("-", format!("{}: File already exists, skipping", &remote_file.name));
    return Ok(());
  }
  let download_url = remote_file.download_url.clone().unwrap();
  let res = client.get(download_url).send()?;
  match res.status().is_success() {
    true => {
      let bytes = res.bytes()?;
      let mut file = std::fs::File::create(&remote_file.name)?;
      let mut content =  Cursor::new(bytes);
      std::io::copy(&mut content, &mut file)?;
      sp.stop_and_persist("✔", format!("{}: Download successful ({:.2}KB, {}ms)", &remote_file.name, (file.metadata()?.len() as f64 / 1024.0), start.elapsed().as_millis()));
    },
    false => sp.stop_and_persist("✕", format!("{}: Download failed ({})", &remote_file.name, res.status().to_string())),
  }

  return Ok(());
}

fn fetch_files(client: &Client, user: &String, repo: &String, path: &String) -> Result<Vec<Content>> {
  let mut sp = Spinner::new(
      Spinners::Dots,
      "Fetching repository".into(),
  );

  let url = ["https://api.github.com/repos", user, repo, "contents", path].join("/");
  let res = client
    .get(url)
    .send()?;

  if !res.status().is_success() {
    sp.stop_and_persist("✕", format!("An error occurred ({})", res.status().to_string()));
    std::process::exit(1);
  }

  let contents: Vec<Content> = res.json()?;
  let files: Vec<Content> = contents.into_iter().filter(|content| content.download_url.is_some()).collect();
  sp.stop_and_persist("✔", format!("Found {} files", files.len()).to_string());
  println!();

  return Ok(files);
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
