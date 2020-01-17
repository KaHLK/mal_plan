use std::error::Error;
use std::time::SystemTime;

use mal_plan::manga;
use mal_plan::{
    read_handled_items, write_handled_items, Cache, Config, HandledHow, HandledItem, InputOptions,
    Item, ListType, Options, Sort,
};

use console::Term;
use directories::ProjectDirs;
use indicatif::{ProgressBar, ProgressStyle};

const MAX_CACHE_AGE: u64 = 60 * 60 * 24 * 3;

fn main() -> Result<(), Box<dyn Error>> {
    let mut options = InputOptions::from_args()?;

    if options.help {
        // TODO: Impl
        return Ok(());
    }

    let term = Term::stdout();

    let project_dirs = match ProjectDirs::from("com", "kahlk", "mal_plan") {
        Some(dir) => dir,
        None => return Err("Failed to load config directory")?,
    };
    let config_dir = project_dirs.config_dir();
    let cache_dir = project_dirs.cache_dir();
    let data_dir = project_dirs.data_dir();

    let config = Config::read(config_dir);

    if let Some(config) = config {
        if options.user.is_none() {
            options.user = Some(config.user);
        }
    }

    if options.user.is_none() {
        term.write_line("Username must be specified. Please enter a username:")?;
        loop {
            match term.read_line() {
                Ok(input) => {
                    options.user = Some(input);
                    break;
                }
                Err(e) => {
                    term.write_str(&format!(
                        "An error occurred parsing your input {}\nPlease try again:",
                        e
                    ))?;
                }
            }
        }
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
    let cache = Cache::read(cache_dir, file_prefix);
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;

    let handled = read_handled_items(data_dir);

    // Check if cache has gotten stale
    let (list, mut handled) = if let Some(cache) = cache
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
        (cache.list, handled)
    } else {
        // Cache has gotten stale so fetch new list (with fancy spinner to show that we are not frozen)
        let spinner = ProgressBar::new_spinner();
        spinner.enable_steady_tick(120);
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template(&format!(
                    "[{{spinner:.green}}] Fetching list for {} with offset {{msg}}",
                    options.user
                )),
        );
        let list: Vec<Item> = match options.list {
            ListType::Manga => manga::fetch_all(options.user.clone(), |offset| {
                spinner.set_message(&format!("{}", offset))
            })?
            .iter()
            .map(|m| m.into())
            .collect(),
            ListType::Anime => unimplemented!(),
        };
        spinner.finish_with_message(&format!("Finished loading {} items", list.len()));
        let handled: Vec<HandledItem> = handled
            .into_iter()
            .filter(|h| {
                if h.item_type != options.list {
                    true
                } else {
                    for i in 0..list.len() {
                        if h.item_id == list[i].id {
                            return true;
                        }
                    }
                    false
                }
            })
            .collect();
        let mut list: Vec<Item> = list
            .into_iter()
            .filter(|item| item.publishing_status == 2)
            .filter(|item| {
                for i in 0..handled.len() {
                    if item.id == handled[i].item_id {
                        return false;
                    }
                }
                true
            })
            .collect();
        list.sort_by_key(|i| i.amount * Sort::Desc.value());
        (list, handled)
    };

    let mut quitting = false;
    let mut remaining: Vec<Item> = vec![];
    let mut clear = 0;
    for item in list {
        // TODO: Make printing prettier?
        if quitting {
            remaining.push(item);
            continue;
        }

        loop {
            term.write_line(&format!(
                "Current {list:?}: {amount:>4} | {id:>7} | {title}",
                list = options.list,
                amount = item.amount,
                title = item.title,
                id = item.id,
            ))?;
            term.write_line("\nWhat do you want to do? (d/e/n/s/h/q)")?;
            clear += 3;
            match term.read_char()? {
                'd' => {
                    handled.push(item.handle(HandledHow::Added));
                    break;
                }
                'e' => {
                    handled.push(item.handle(HandledHow::NotFound));
                    break;
                }
                'n' => {
                    handled.push(item.handle(HandledHow::NotFinished));
                    break;
                }
                's' => {
                    remaining.push(item);
                    break;
                }
                'h' => {
                    term.clear_last_lines(clear)?;
                    term.write_line(
                        "
You can do the following:
    d: Mark the current item as 'downloaded'
    e: Mark the current item as 'not found'
    n: Mark the current item as 'not finished'
    s: Skip the current item
    h: Display the current message

    q: Quit
",
                    )?;
                    clear = 10;
                    continue;
                }
                'q' => {
                    quitting = true;
                    break;
                }
                v => {
                    term.clear_last_lines(clear)?;
                    term.write_line(&format!("Unknown input '{}'. Press 'h' for help", v))?;
                    clear = 1;
                }
            }
        }

        term.clear_last_lines(clear)?;
        clear = 0;
    }

    write_handled_items(data_dir, &handled)?;
    let cache = Cache::new(now, options.user, remaining);
    cache.write(cache_dir, file_prefix)?;
    Ok(())
}
