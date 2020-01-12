use std::error::Error;
use std::io;
use std::time::SystemTime;

use mal_plan::manga;
use mal_plan::{Cache, Config, InputOptions, Item, ListType, Options, Sort};

use directories::ProjectDirs;
use indicatif::{ProgressBar, ProgressStyle};

const MAX_CACHE_AGE: u64 = 60 * 60 * 24 * 3;

fn main() -> Result<(), Box<dyn Error>> {
    let mut options = InputOptions::from_args()?;

    if options.help {
        // TODO: Impl
        return Ok(());
    }

    let project_dirs = match ProjectDirs::from("com", "kahlk", "mal_plan") {
        Some(dir) => dir,
        None => return Err("Failed to load config directory")?,
    };

    let config_dir = project_dirs.config_dir();
    let config = Config::read(config_dir);

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
        options.user = Some(String::from(input.trim()));
    }

    let options: Options = options.into();

    if options.save {
        let mut config = Config::new();
        config.user = options.user.clone();
        config.write(config_dir)?;
    }

    let file_prefix = match options.list {
        ListType::Manga => "manga",
        ListType::Anime => "anime",
    };
    let cache_dir = project_dirs.cache_dir();
    let cache = Cache::read(cache_dir, file_prefix);
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    // TODO: Load already handled items from files (added, not-found, not-finished)

    // Check if cache has gotten stale
    let list = if let Some(cache) = cache
        .filter(|c| c.user == options.user)
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
            ListType::Manga => manga::fetch_all(options.user.clone(), Sort::Desc, |offset| {
                spinner.set_message(&format!("{}", offset))
            })?
            .iter()
            .map(|m| m.into())
            .collect(),
            ListType::Anime => unimplemented!(),
        };
        spinner.finish_with_message(&format!("Finished loading {} items", list.len()));
        list
        // TODO: Remove already handled items from list
        // TODO: Remove handled items that no longer exists in list
    };

    // TODO: Loop over list and get user input for each item
    // TODO: Save handled items

    let cache = Cache::new(now, options.user, list);
    cache.write(cache_dir, file_prefix)?;
    Ok(())
}
