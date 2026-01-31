//!
//! Pull code out of Markdown documentation for testing
//!

use argh::FromArgs;
use color_eyre::eyre::Result;

mod codechunk;
mod codefile;
mod codegrab;
mod validate;

#[derive(FromArgs)]
/// Pull codeblocks from Markdown into files for testing
struct CodeMdArgs {
    #[argh(subcommand)]
    command: Option<SubCommand>,

    /// file to pull code from (shorthand for 'run -f <file>')
    #[argh(option, short = 'f')]
    filename: Option<String>,

    /// output directory for code files
    #[argh(option, short = 'o')]
    output: Option<String>,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum SubCommand {
    Run(RunCommand),
    Validate(ValidateCommand),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "run")]
/// Extract code blocks from markdown and write to files
struct RunCommand {
    /// file to pull code from
    #[argh(option, short = 'f')]
    filename: String,

    /// output directory for code files
    #[argh(option, short = 'o')]
    output: Option<String>,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "validate")]
/// Validate codemd syntax without writing files
struct ValidateCommand {
    /// file to validate
    #[argh(option, short = 'f')]
    filename: String,

    /// show verbose output including warnings
    #[argh(switch, short = 'v')]
    verbose: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args: CodeMdArgs = argh::from_env();

    match args.command {
        Some(SubCommand::Run(run)) => {
            run_extract(&run.filename, run.output.as_deref())?;
        }
        Some(SubCommand::Validate(val)) => {
            run_validate(&val.filename, val.verbose)?;
        }
        None => {
            // Backwards compatibility: if -f is provided without subcommand, run extraction
            if let Some(filename) = args.filename {
                run_extract(&filename, args.output.as_deref())?;
            } else {
                // Print help
                eprintln!("Usage: codemd <command> [options]");
                eprintln!();
                eprintln!("Commands:");
                eprintln!("  run       Extract code blocks from markdown and write to files");
                eprintln!("  validate  Validate codemd syntax without writing files");
                eprintln!();
                eprintln!("Options:");
                eprintln!("  -f <file>  File to process (shorthand for 'run -f <file>')");
                eprintln!("  -o <dir>   Output directory for code files");
                eprintln!();
                eprintln!("Examples:");
                eprintln!("  codemd -f example.md              # Extract to current directory");
                eprintln!("  codemd run -f example.md -o out/  # Extract to out/ directory");
                eprintln!("  codemd validate -f example.md     # Validate syntax only");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

/// Run the extraction process: validate first, then extract and write files
fn run_extract(filename: &str, output: Option<&str>) -> Result<()> {
    // Read the provided file
    let md = std::fs::read_to_string(filename)?;
    let lines: Vec<String> = md.lines().map(str::to_string).collect();

    // Validate before performing any file operations
    let validation = validate::validate_markdown_lines(&lines)?;

    // Print any warnings
    for warning in &validation.warnings {
        eprintln!("Warning: {}", warning.message);
    }

    // Parse and write files
    let codeblocks = codegrab::find_codeblocks(&lines)?;
    let files = codefile::CodeFile::from_blocks(&codeblocks)?;

    let default_filename = "default.out".to_string();
    let output_dir = output.unwrap_or(".");

    for cf in &files {
        cf.write_to_file(output_dir, &default_filename)?;
    }

    Ok(())
}

/// Run validation only, without writing any files
fn run_validate(filename: &str, verbose: bool) -> Result<()> {
    let result = validate::validate_markdown_file(filename)?;

    println!("Validation successful!");
    println!("  Code blocks: {}", result.block_count);
    println!("  Output files: {}", result.file_count);

    if verbose {
        println!();
        println!("Files to be written:");
        for file in &result.files {
            println!("  - {}", file.as_deref().unwrap_or("default.out"));
        }

        if !result.warnings.is_empty() {
            println!();
            println!("Warnings:");
            for warning in &result.warnings {
                println!("  Block {}: {}", warning.block_index, warning.message);
            }
        }
    } else if !result.warnings.is_empty() {
        println!("  Warnings: {} (use -v for details)", result.warnings.len());
    }

    Ok(())
}
