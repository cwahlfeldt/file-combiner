use clap::Parser;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Combines text files from a directory into a single output file"
)]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,

    #[arg(short = 'x', long)]
    ignore: Option<String>,
}

fn should_ignore(path: &Path, ignore_list: &Option<Vec<String>>) -> bool {
    if let Some(ignore_patterns) = ignore_list {
        let path_str = path.to_string_lossy();
        for pattern in ignore_patterns {
            if path_str.contains(pattern) {
                return true;
            }
        }
    }
    false
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let input_dir = PathBuf::from(&args.input);
    let output_file = &args.output;

    if let Some(parent) = Path::new(output_file).parent() {
        fs::create_dir_all(parent)?;
    }

    let ignore_list = args
        .ignore
        .map(|i| i.split(',').map(String::from).collect());

    if !input_dir.exists() {
        eprintln!("Error: Input directory does not exist");
        std::process::exit(1);
    }

    let mut writer = File::create(output_file)?;

    writeln!(writer, "Directory Structure:\n")?;
    print_tree(&input_dir, "", true, &mut writer, &ignore_list)?;
    writeln!(writer, "\nFile Contents:\n")?;

    for entry in WalkDir::new(&input_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !should_ignore(e.path(), &ignore_list))
    {
        let path = entry.path();
        if path.is_file() {
            let relative_path = path
                .strip_prefix(&input_dir)
                .unwrap_or(path)
                .display()
                .to_string();

            writeln!(writer, "=== {} ===\n", relative_path)?;
            let contents = fs::read_to_string(path)?;
            writeln!(writer, "{}\n", contents)?;
        }
    }

    println!("Successfully combined files into: {}", output_file);
    Ok(())
}

fn print_tree(
    path: &Path,
    prefix: &str,
    is_last: bool,
    writer: &mut File,
    ignore_list: &Option<Vec<String>>,
) -> io::Result<()> {
    if should_ignore(path, ignore_list) {
        return Ok(());
    }

    let display = path
        .file_name()
        .unwrap_or_else(|| path.as_os_str())
        .to_string_lossy();

    writeln!(
        writer,
        "{}{}{}",
        prefix,
        if is_last { "└── " } else { "├── " },
        display
    )?;

    if path.is_dir() {
        let mut entries = fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .filter(|e| !should_ignore(&e.path(), ignore_list))
            .collect::<Vec<_>>();

        entries.sort_by_key(|e| e.path());

        for (i, entry) in entries.iter().enumerate() {
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            print_tree(
                &entry.path(),
                &new_prefix,
                i == entries.len() - 1,
                writer,
                ignore_list,
            )?;
        }
    }
    Ok(())
}
