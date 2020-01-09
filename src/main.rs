use mal_plan::Options;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::from_args()?;

    println!("options: {:?}", options);
    if options.help {
        // TODO: Impl
        return Ok(());
    }

    // TODO: Load from config file
    // TODO: Check if user is still missing and return error if so

    // TODO: Cache lists fetched from mal

    Ok(())
}
