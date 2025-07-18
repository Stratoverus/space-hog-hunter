use std::collections::HashMap;
use std::io::{self, Write};
use walkdir::WalkDir;
use std::path::Path;

fn main() {
    println!("Enter the directory to analyze:");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let dir = input.trim();

    if !Path::new(dir).is_dir() {
        println!("{} is not a valid directory.", dir);
        return;
    }

    let mut largest_files = Vec::new();
    let mut ext_count: HashMap<String, usize> = HashMap::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            largest_files.push((entry.path().display().to_string(), size));

            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                *ext_count.entry(ext.to_string()).or_insert(0) += 1;
            }
        }
    }

    largest_files.sort_by(|a, b| b.1.cmp(&a.1));

    loop {
        println!("\nMenu:");
        println!("1. Show top 10 largest files");
        println!("2. Show most common file extension");
        println!("3. Exit");
        print!("Enter your choice: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read choice");
        match choice.trim() {
            "1" => {
                println!("\nTop 10 largest files:");
                for (file, size) in largest_files.iter().take(10) {
                    println!("{} - {}", file, human_readable_size(*size));
                }
            },
            "2" => {
                if let Some((most_common_ext, count)) = ext_count.iter().max_by_key(|entry| entry.1) {
                    println!("\nMost common file extension: {} ({} files)", most_common_ext, count);
                } else {
                    println!("\nNo file extensions found.");
                }
            },
            "3" => {
                println!("Exiting.");
                break;
            },
            _ => println!("Invalid choice. Please enter 1, 2, or 3."),
        }
    }
}

//Converting function to convert bytes to something more reasonable.
fn human_readable_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    match bytes {
        b if b >= TB => format!("{:.2} TB", b as f64 / TB as f64),
        b if b >= GB => format!("{:.2} GB", b as f64 / GB as f64),
        b if b >= MB => format!("{:.2} MB", b as f64 / MB as f64),
        b if b >= KB => format!("{:.2} KB", b as f64 / KB as f64),
        _ => format!("{} bytes", bytes),
    }
}
