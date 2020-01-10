use mal_plan::manga;
use mal_plan::{Config, Item, ListType, Options, Sort};

use std::error::Error;
use std::fs;
use std::io;

use directories::ProjectDirs;
use indicatif::{ProgressBar, ProgressStyle};

fn main() -> Result<(), Box<dyn Error>> {
    let mut options = Options::from_args()?;

    if options.help {
        // TODO: Impl
        return Ok(());
    }

    let project_dirs = match ProjectDirs::from("com", "kahlk", "mal_plan") {
        Some(dir) => dir,
        None => return Err("Failed to load config directory")?,
    };

    let config_path = project_dirs.config_dir().join("config");
    let config: Option<Config> = fs::read_to_string(&config_path)
        .map(|s| serde_json::from_str(&s[..]).ok())
        .ok()
        .flatten();

    if let Some(config) = config {
        if options.user.is_none() {
            options.user = Some(config.user);
        }
    }

    if options.user.is_none() {
        println!("Username must be specified. Please enter a username:");
        let mut input = String::new();
        loop {
            match io::stdin().read_line(&mut input) {
                Err(e) => println!(
                    "An error occurred parsing your input {}\nSuccessfully read: {}\nPlease try again:",
                    e, input
                ),
                _ => break,
            }
        }
        options.user = input.lines().next().map(|s| String::from(s));
    }

    if options.save {
        let mut config = Config::new();

        config.user = options.user.clone().unwrap();

        let s = serde_json::to_string_pretty(&config)
            .unwrap_or_else(|e| panic!("Error occurred saving config file: {:?}", e));
        fs::write(config_path, s)
            .unwrap_or_else(|e| panic!("Error occurred saving config file: {:?}", e));
    }

    // TODO: Read cached list

    let spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(120);
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("[{spinner:.green}] Loading with offset {msg}"),
    );
    // TODO: Only fetch list from mal if cache is old or skippped
    let list: Vec<Item> = match options.list {
        ListType::Manga => manga::fetch_all(options.user.unwrap(), Sort::Desc, |offset| {
            spinner.set_message(&format!("{}", offset))
        })?
        .iter()
        .map(|m| m.into())
        .collect(),
        ListType::Anime => unimplemented!(),
    };
    spinner.finish_with_message(&format!("Finished loading {} items", list.len()));

    // TODO: Cache list fetched from mal (only if new list was fetched)
    // TODO: Load already handled items from files (added, not-found, not-finished)
    // TODO: Remove already handled items from list
    // TODO: Remove handled items that no longer exists in list
    // TODO: Loop over list and get user input for each item
    // TODO: Save handled items

    Ok(())
}
