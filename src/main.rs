use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use semver::{Version, VersionReq};
use std::env;
use std::io;
use std::process;

mod preprocessor;

fn main() {
    let chapter_lister = preprocessor::ChapterList::new();

    let mut args = env::args().skip(1);
    if args.next().as_deref() == Some("supports") {
        let renderer = args.next().unwrap_or_default();
        handle_supports(&chapter_lister, &renderer);
    } else if let Err(error) = handle_preprocessing(&chapter_lister) {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let version = Version::parse(&ctx.mdbook_version);
    let version_req = VersionReq::parse(mdbook::MDBOOK_VERSION);
    if let (Ok(version), Ok(version_req)) = (version, version_req) {
        if !version_req.matches(&version) {
            eprintln!(
                "Warning: The {} plugin was built against version {} of mdbook, \
                 but we're being called from version {}",
                pre.name(),
                mdbook::MDBOOK_VERSION,
                ctx.mdbook_version
            );
        }
    } else {
        eprintln!(
            "Warning: The {} plugin could not compare mdbook versions: built against {}, called from {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, renderer: &str) -> ! {
    if pre.supports_renderer(renderer) {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
