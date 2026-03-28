use std::io;
use std::process;

use anyhow::Result;
use mdbook_preprocessor::{parse_input, Preprocessor};
use mdbook_beans::BeansPreprocessor;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() >= 3 && args[1] == "supports" {
        let renderer = &args[2];
        if BeansPreprocessor.supports_renderer(renderer)? {
            process::exit(0);
        } else {
            process::exit(1);
        }
    }

    let (ctx, book) = parse_input(io::stdin())?;
    let processed = BeansPreprocessor.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed)?;

    Ok(())
}
