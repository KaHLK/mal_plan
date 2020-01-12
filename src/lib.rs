use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

pub mod manga;

use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "config.json";
const CACHE_FILE: &str = "cache.json";
const HANDLED_FILE: &str = "handled.json";

type Result<T> = std::result::Result<T, String>;

#[derive(Debug)]
pub struct InputOptions {
    pub save: bool,
    pub user: Option<String>,
    pub list: ListType,
    pub help: bool,
    // TODO: Add the following options: no-cache, cache-age, handled-file, config-file, ignore-config
    // TODO: Add commands to take another look at not-found & not-finished
    // TODO: Add commands to sort ascending (default sort will be descending)
}

impl<'a> InputOptions {
    pub fn from_args() -> Result<InputOptions> {
        let mut options = InputOptions {
            save: false,
            user: None,
            list: ListType::Manga,
            help: false,
        };

        let mut args = env::args().into_iter().skip(1);
        while let Some(arg) = args.next() {
            match &arg[..] {
                "--help" => options.set_help(),
                "--save" => options.set_save(),
                "--user" => {
                    if let Some(user) = args.next() {
                        options.user = Some(user);
                    }
                }
                "--list" => {
                    if let Some(list) = args.next() {
                        options.list = ListType::from_str(&list[..])?;
                    }
                }

                "--manga" => options.list = ListType::Manga,
                "--anime" => options.list = ListType::Anime,

                v => {
                    if v.starts_with("-") {
                        for c in v.chars().skip(1) {
                            match c {
                                'h' => options.set_help(),
                                's' => options.set_save(),
                                _ => return Err(String::from(Error::ArgumentError(arg))),
                            }
                        }
                    } else {
                        return Err(String::from(Error::ArgumentError(arg)));
                    }
                }
            }
        }

        Ok(options)
    }

    fn set_help(&mut self) {
        self.help = true;
    }

    fn set_save(&mut self) {
        self.save = true;
    }
}

pub struct Options {
    pub save: bool,
    pub user: String,
    pub list: ListType,
}

impl From<InputOptions> for Options {
    fn from(input: InputOptions) -> Self {
        Options {
            save: input.save,
            user: input.user.unwrap(),
            list: input.list,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ListType {
    Manga,
    Anime,
}

impl FromStr for ListType {
    type Err = String;

    fn from_str(s: &str) -> Result<ListType> {
        match &s.to_lowercase()[..] {
            "manga" => Ok(ListType::Manga),
            "anime" => Ok(ListType::Anime),
            val => Err(String::from(Error::ListError(String::from(val)))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub user: String,
    // TODO: Add fields: cache-age, handled-file
}

impl Config {
    pub fn new() -> Config {
        Config {
            user: String::from(""),
        }
    }

    pub fn read(dir: &Path) -> Option<Config> {
        read_file(dir, CONFIG_FILE).and_then(|s| de(&s)).ok()
    }

    pub fn write(&self, dir: &Path) -> Result<()> {
        se(self).and_then(|s| write_file(dir, CONFIG_FILE, s))
    }
}

pub enum Error {
    ArgumentError(String),
    ListError(String),
    FileError(PathBuf, io::Error),
    FileReadError(PathBuf, io::Error),
    FileWriteError(PathBuf, io::Error),
    SerdeDeError(serde_json::Error),
    SerdeSerError(serde_json::Error),
}

impl From<Error> for String {
    fn from(err: Error) -> Self {
        match err {
            Error::ArgumentError(val) => {
                format!("Unknown argument {}. Use --help to see all options", val)
            }
            Error::ListError(val) => {
                format!("Unknown list type {}. allowed values: manga, anime", val)
            }
            Error::FileError(val, err) => format!(
                "An error occurred when trying to interact with file '{:?}'. {}",
                val.to_str(),
                err
            ),
            Error::FileReadError(val, err) => format!(
                "An error occurred when trying to read from file '{:?}'. {}",
                val.to_str(),
                err
            ),
            Error::FileWriteError(val, err) => format!(
                "An error occurred when trying to write to file '{:?}'. {}",
                val.to_str(),
                err
            ),
            Error::SerdeDeError(err) => {
                format!("An error occurred when trying to deserialize: {:?}", err)
            }
            Error::SerdeSerError(err) => {
                format!("An error occurred when trying to serialize: {:?}", err)
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    pub item_type: ListType,
    pub id: u32,
    pub amount: u16,
    pub publishing_status: u8,
    pub url: String,
    pub media_type: ItemMediaType,
}

impl Item {
    pub fn handle(self, how: HandledHow) -> HandledItem {
        HandledItem {
            item_id: self.id,
            item_type: self.item_type,
            how,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ItemMediaType {
    Manga,
    Novel,
    OneShot,
    Doujinshi,
    Manhwa,
    Manhua,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HandledItem {
    pub item_id: u32,
    pub item_type: ListType,
    pub how: HandledHow,
}

pub fn read_handled_items(dir: &Path) -> Vec<HandledItem> {
    read_file(dir, HANDLED_FILE)
        .and_then(|s| de(&s))
        .unwrap_or(vec![])
}

pub fn write_handled_items(dir: &Path, content: &Vec<HandledItem>) -> Result<()> {
    se(content).and_then(|s| write_file(dir, HANDLED_FILE, s))
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HandledHow {
    Added,
    NotFound,
    NotFinished,
}

pub enum Sort {
    Asc,
    Desc,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cache {
    pub fetched_at: Duration,
    pub user: String,
    pub list: Vec<Item>,
}

impl Cache {
    pub fn new(fetched_at: Duration, user: String, list: Vec<Item>) -> Self {
        Cache {
            fetched_at,
            list,
            user,
        }
    }

    pub fn read(dir: &Path, prefix: &str) -> Option<Self> {
        let file = Self::get_file(prefix);
        read_file(dir, &file).and_then(|s| de(&s)).ok()
    }

    pub fn write(&self, dir: &Path, prefix: &str) -> Result<()> {
        let file = Self::get_file(prefix);
        se(self).and_then(|s| write_file(dir, &file, s))
    }

    fn get_file(prefix: &str) -> String {
        format!("{}_{}", prefix, CACHE_FILE)
    }
}

fn read_file(dir: &Path, file: &str) -> Result<String> {
    let path = dir.join(file);
    fs::read_to_string(&path).map_err(|e| String::from(Error::FileReadError(path, e)))
}

fn write_file(dir: &Path, file: &str, content: String) -> Result<()> {
    let path = dir.join(file);
    fs::create_dir_all(&dir).map_err(|e| String::from(Error::FileError(path.clone(), e)))?;

    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .map_err(|e| String::from(Error::FileError(path.clone(), e)))?;
    f.write_all(content.as_bytes())
        .map_err(|e| String::from(Error::FileWriteError(path.clone(), e)))?;
    Ok(())
}

fn de<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    serde_json::from_str::<T>(s).map_err(|e| String::from(Error::SerdeDeError(e)))
}

fn se<T>(v: &T) -> Result<String>
where
    T: Serialize,
{
    serde_json::to_string(v).map_err(|e| String::from(Error::SerdeSerError(e)))
}
