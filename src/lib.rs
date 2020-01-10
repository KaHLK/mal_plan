use std::env;
use std::str::FromStr;

pub mod manga;

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Options {
    pub save: bool,
    pub user: Option<String>,
    pub list: ListType,
    pub help: bool,
    // TODO: Add the following options: no-cache, cache-age, not-found-file, not-finished-file, added-file, config-file, ignore-config
    // TODO: Add commands to take another look at not-found & not-finished
    // TODO: Add commands to sort ascending (default sort will be descending)
}

impl<'a> Options {
    pub fn from_args() -> Result<Options, String> {
        let mut options = Options {
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
                                _ => return Err(Error::ArgumentError(arg).to_string()),
                            }
                        }
                    } else {
                        return Err(Error::ArgumentError(arg).to_string());
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

#[derive(Debug, Serialize, Deserialize)]
pub enum ListType {
    Manga,
    Anime,
}

impl FromStr for ListType {
    type Err = String;

    fn from_str(s: &str) -> Result<ListType, Self::Err> {
        match &s.to_lowercase()[..] {
            "manga" => Ok(ListType::Manga),
            "anime" => Ok(ListType::Anime),
            val => Err(Error::ListError(String::from(val)).to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub user: String,
    // TODO: Add fields: cache-age, not-found-file, not-finished-file, added-file
}

impl Config {
    pub fn new() -> Config {
        Config {
            user: String::from(""),
        }
    }
}

pub enum Error {
    ArgumentError(String),
    ListError(String),
}

impl<'a> ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Error::ArgumentError(val) => {
                format!("Unknown argument {}. Use --help to see all options", val)
            }
            Error::ListError(val) => {
                format!("Unknown list type {}. allowed values: manga, anime", val)
            }
        }
    }
}

pub struct Item {
    pub item_type: ItemType,
    pub id: u32,
    pub amount: u16,
    pub publishing_status: u8,
    pub url: String,
    pub media_type: ItemMediaType,
}

pub enum ItemType {
    Manga,
    Anime,
}

pub enum ItemMediaType {
    Manga,
    Novel,
    OneShot,
    Doujinshi,
    Manhwa,
    Manhua,
}

pub trait IntoItem {
    fn into_item(self) -> Item;
}

pub enum Sort {
    Asc,
    Desc,
}
