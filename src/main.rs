use std::error::Error;
use std::io;
use std::time::SystemTime;

use mal_plan::manga;
use mal_plan::{read_file, write_file, Cache, Config, Item, ListType, Options, Sort};

use directories::ProjectDirs;
use indicatif::{ProgressBar, ProgressStyle};

const CONFIG_FILE: &str = "config.json";
const CACHE_FILE: &str = "cache.json";
const MAX_CACHE_AGE: u64 = 60 * 60 * 24 * 3;

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

    let config_dir = project_dirs.config_dir();
    let config: Option<Config> = read_file(config_dir, CONFIG_FILE)
        .map(|s| serde_json::from_str(&s).ok())
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

        let s = serde_json::to_string(&config)
            .unwrap_or_else(|e| panic!("Error occurred saving config file: {:?}", e));
        // TODO: don't panic
        write_file(config_dir, CONFIG_FILE, s)?;
    }

    let file_prefix = match options.list {
        ListType::Manga => "manga",
        ListType::Anime => "anime",
    };
    let cache_dir = project_dirs.cache_dir();
    let cache_file = format!("{}_{}", file_prefix, CACHE_FILE);
    let cache: Option<Cache> = read_file(cache_dir, &cache_file)
        .map(|s| serde_json::from_str(&s).ok())
        .ok()
        .flatten();
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;

    // Check if cache has gotten stale
    let list = if let Some(cache) = cache
        // TODO: Also check if username is the same as the run requested in this run (save to different cache files?)
        .map(|c| now.checked_sub(c.fetched_at).map(|diff| (c, diff)))
        .flatten()
        .and_then(|(c, diff)| {
            if diff.as_secs() > MAX_CACHE_AGE {
                None
            } else {
                Some(c)
            }
        }) {
        // Cache is still fresh so use list from cache
        cache.list
    } else {
        // Cache has gotten stale so fetch new list (with fancy spinner to show that we are not frozen)
        let spinner = ProgressBar::new_spinner();
        spinner.enable_steady_tick(120);
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("[{spinner:.green}] Loading with offset {msg}"),
        );
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
        list
    };

    // TODO: Load already handled items from files (added, not-found, not-finished)
    // TODO: Remove already handled items from list
    // TODO: Remove handled items that no longer exists in list
    // TODO: Loop over list and get user input for each item
    // TODO: Save handled items

    let cache = Cache::new(now, list);
    let s = serde_json::to_string(&cache)?;
    write_file(cache_dir, &cache_file, s)?;
    Ok(())
}
