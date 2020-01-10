use std::env;
use std::str::FromStr;

#[derive(Debug)]
pub struct Options {
    pub interactive: bool,
    pub save: bool,
    pub user: Option<String>,
    pub list: ListType,
    pub help: bool,
}

impl<'a> Options {
    pub fn from_args() -> Result<Options, String> {
        let mut options = Options {
            interactive: false,
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
                "--interactive" => options.set_interactive(),
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
                                'i' => options.set_interactive(),
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

    fn set_interactive(&mut self) {
        self.interactive = true;
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
