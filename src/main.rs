//!
//! Pull code out of Markdown documentation for testing
//!

use argh::FromArgs;
use color_eyre::eyre::Result;

mod codechunk;
mod codefile;
mod codegrab;



#[derive(FromArgs)]
/// Pull codeblocks from Markdown into files for testing
struct CodeMdArgs {
    /// file to pull code from
    #[argh(option, short = 'f')]
    filename: String,
    /// output directory for code files
    #[argh(option, short = 'o')]
    output: Option<String>,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args: CodeMdArgs = argh::from_env();

    // Read the provided file
    let md = std::fs::read_to_string(&args.filename)?;
    let lines = md.lines().map(str::to_string).collect::<Vec<String>>();
    let codeblocks = codegrab::find_codeblocks(&lines)?;
    let files = codefile::CodeFile::from_blocks(&codeblocks)?;

    let default_filename = "default.out".to_string();
    let default_outdir = ".".to_string();
    let output_dir = args.output.as_ref().unwrap_or(&default_outdir);

    files.iter().for_each(|cf| {
        cf.write_to_file(output_dir, &default_filename).unwrap();
    });

    Ok(())
}

