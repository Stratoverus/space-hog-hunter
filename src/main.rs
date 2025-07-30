use std::io::{self, Write};
use walkdir::WalkDir;
use std::path::Path;

//Main function and loop for program
fn main() {
    loop {
        println!("\nMenu:");
        println!("1. Analyze a specific Directory");
        println!("2. Analyze Common Large Directories(e.g., Downloads, Documents, Videos, AppData...)");
        println!("3. Scan an entire drive (WARNING: may be slow and require admin permissions)");
        println!("4. Check large known game directories (Steam, Epic, etc.)");
        println!("5. Find and list largest 'useless' files (temp, log, cache, etc.)");
        println!("6. Exit");
        print!("Enter your choice: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read choice");
        match choice.trim() {
            "1" => {
                let dir = prompt_directory();
                analyze_largest_files(&dir);
            },
            "2" => {
                println!("\nAnalyzing common large directories...");
                let user_profile = std::env::var("USERPROFILE").unwrap_or_else(|_| String::from("C:/Users/Public"));
                let common_dirs = vec![
                    format!("{}/Downloads", user_profile),
                    format!("{}/Documents", user_profile),
                    format!("{}/Videos", user_profile),
                    format!("{}/Pictures", user_profile),
                    format!("{}/Music", user_profile),
                    String::from("C:/Users/Public/Documents"),
                    String::from("C:/Users/Public/Downloads"),
                    String::from("C:/Users/Public/Videos"),
                    String::from("C:/Users/Public/Pictures"),
                    String::from("C:/Users/Public/Music"),
                    String::from("C:/Users/Public/AppData")
                ];
                let mut all_subdirs: Vec<(String, u64)> = Vec::new();
                for dir in &common_dirs {
                    let subdirs = get_largest_subdirectories(dir, usize::MAX);
                    all_subdirs.extend(subdirs);
                }
                if all_subdirs.is_empty() {
                    println!("No subdirectories found or no files in common directories.");
                } else {
                    all_subdirs.sort_by(|a, b| b.1.cmp(&a.1));
                    println!("\nTop 10 largest subdirectories across common directories:");
                    for (i, (subdir, size)) in all_subdirs.iter().take(10).enumerate() {
                        println!("{}. {} - {}", i + 1, subdir, human_readable_size(*size));
                    }
                    investigation_loop(&all_subdirs);
                }
            },
            "3" => {
                let available_drives = list_available_drives();
                if available_drives.is_empty() {
                    println!("No drives found.");
                    return;
                }
                println!("\nAvailable drives:");
                for (letter, _) in &available_drives {
                    print!("{} ", letter);
                }
                println!("");
                println!("Enter the drive letter to scan:");
                let mut drive_input = String::new();
                io::stdin().read_line(&mut drive_input).expect("Failed to read drive letter");
                let drive = drive_input.trim().to_uppercase();
                let drive_path = format!("{}:/", drive);
                if !available_drives.iter().any(|(l, _)| l.to_string() == drive) {
                    println!("Drive {} not found.", drive);
                    return;
                }
                println!("Scanning entire {} drive. This may take a long time and require admin permissions.", drive);
                analyze_largest_files(&drive_path);
            },
            "4" => {
                println!("\nCheck large known game directories (Steam, Epic, etc.)");
                println!("Scan only C: drive for games, or scan all available drives? (Enter 'c' for C: only, 'a' for all drives)");
                let mut scan_choice = String::new();
                io::stdin().read_line(&mut scan_choice).expect("Failed to read input");
                let scan_choice = scan_choice.trim().to_lowercase();
                let game_dirs = vec![
                    "Program Files/Epic Games",
                    "Program Files (x86)/Origin Games",
                    "Program Files (x86)/Ubisoft/Ubisoft Game Launcher/games"
                ];
                let mut all_subdirs: Vec<(String, u64)> = Vec::new();
                // --- Steam library detection ---
                let steam_vdf = "C:/Program Files (x86)/Steam/steamapps/libraryfolders.vdf";
                let mut steam_libraries = Vec::new();
                if Path::new(steam_vdf).is_file() {
                    if let Ok(contents) = std::fs::read_to_string(steam_vdf) {
                        for line in contents.lines() {
                            let line = line.trim();
                            // Look for lines like: "1"    "D:\\SteamLibrary"
                            if line.starts_with('"') && line.contains("\\") {
                                let parts: Vec<&str> = line.split('"').collect();
                                if parts.len() >= 4 {
                                    let path = parts[3].replace("\\", "/");
                                    steam_libraries.push(path);
                                }
                            }
                        }
                    }
                }
                //Always add default Steam library
                steam_libraries.push(String::from("C:/Program Files (x86)/Steam/steamapps/common"));
                for lib in &steam_libraries {
                    //Always scan only steamapps/common for each Steam library
                    let steamapps_common = if lib.ends_with("common") {
                        lib.clone()
                    } else if lib.ends_with("steamapps") {
                        format!("{}/common", lib.trim_end_matches('/'))
                    } else {
                        format!("{}/steamapps/common", lib.trim_end_matches('/'))
                    };
                    if Path::new(&steamapps_common).is_dir() {
                        let subdirs = get_largest_subdirectories(&steamapps_common, usize::MAX);
                        all_subdirs.extend(subdirs);
                    }
                }
                // --- End Steam library detection ---
                if scan_choice == "a" {
                    //Scan all available drives for other launchers
                    let available_drives = list_available_drives().into_iter().map(|(_, d)| d).collect::<Vec<_>>();
                    if available_drives.is_empty() {
                        println!("No drives found.");
                        return;
                    }
                    for drive in &available_drives {
                        for dir in &game_dirs {
                            let full_path = format!("{}{}", drive, dir);
                            let subdirs = get_largest_subdirectories(&full_path, usize::MAX);
                            all_subdirs.extend(subdirs);
                        }
                    }
                } else {
                    //Default to C: only for other launchers
                    let c_drive = "C:/";
                    for dir in &game_dirs {
                        let full_path = format!("{}{}", c_drive, dir);
                        let subdirs = get_largest_subdirectories(&full_path, usize::MAX);
                        all_subdirs.extend(subdirs);
                    }
                }
                if all_subdirs.is_empty() {
                    println!("No subdirectories found or no files in game directories.");
                } else {
                    //Normalize and deduplicate paths
                    let mut deduped = normalize_and_dedup_paths(&all_subdirs);
                    deduped.sort_by(|a, b| b.1.cmp(&a.1));
                    println!("\nTop 10 largest subdirectories across game directories:");
                    for (i, (subdir, size)) in deduped.iter().take(10).enumerate() {
                        println!("{}. {} - {}", i + 1, subdir, human_readable_size(*size));
                    }
                    //Investigation loop
                    investigation_loop(&deduped);
                }
            },
            "5" => {
                println!("\nScanning common directories for largest 'useless' files...");
                let user_profile = std::env::var("USERPROFILE").unwrap_or_else(|_| String::from("C:/Users/Public"));
                let common_dirs = vec![
                    format!("{}/Downloads", user_profile),
                    format!("{}/Documents", user_profile),
                    format!("{}/Videos", user_profile),
                    format!("{}/Pictures", user_profile),
                    format!("{}/Music", user_profile),
                    String::from("C:/Users/Public/Documents"),
                    String::from("C:/Users/Public/Downloads"),
                    String::from("C:/Users/Public/Videos"),
                    String::from("C:/Users/Public/Pictures"),
                    String::from("C:/Users/Public/Music"),
                    String::from("C:/Users/Public/AppData")
                ];
                let mut all_useless_files: Vec<(String, u64)> = Vec::new();
                for dir in &common_dirs {
                    if std::path::Path::new(dir).is_dir() {
                        let mut files = find_useless_files_in_dir(dir);
                        all_useless_files.append(&mut files);
                    }
                }
                if all_useless_files.is_empty() {
                    println!("No 'useless' files found in common directories.");
                } else {
                    all_useless_files.sort_by(|a, b| b.1.cmp(&a.1));
                    println!("\nTop 10 largest 'useless' files across common directories:");
                    for (file, size) in all_useless_files.iter().take(10) {
                        println!("{} - {}", file, human_readable_size(*size));
                    }
                }
            },
            "6" => {
                println!("Exiting.");
                break;
            },
            _ => println!("Invalid choice. Please enter 1, 2, 3, 4, 5, or 6."),
        }
    }

//Finds and returns the largest 'useless' files in a directory
fn find_useless_files_in_dir(dir: &str) -> Vec<(String, u64)> {
    use std::ffi::OsStr;
    let useless_exts = [
        "tmp", "log", "bak", "old", "dmp", "cache", "temp", "msi", "swp", "chk", "part"
    ];
    let useless_names = [
        ".ds_store", "thumbs.db"
    ];
    let mut files = Vec::new();
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            let ext = path.extension().and_then(OsStr::to_str).map(|e| e.to_ascii_lowercase());
            let fname = path.file_name().and_then(OsStr::to_str).map(|f| f.to_ascii_lowercase());
            let is_useless = ext.as_ref().map_or(false, |e| useless_exts.contains(&e.as_str()))
                || fname.as_ref().map_or(false, |f| useless_names.contains(&f.as_str()));
            if is_useless {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                files.push((path.display().to_string(), size));
            }
        }
    }
    files
}

//Asks what directory to analyze
fn prompt_directory() -> String {
    println!("Enter the directory to analyze:");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let dir = input.trim().to_string();
    if !Path::new(&dir).is_dir() {
        println!("{} is not a valid directory.", dir);
        return prompt_directory();
    }
    dir
}

//Analyze specified directory 
fn analyze_largest_files(dir: &str) {
    if !Path::new(dir).is_dir() {
        println!("{} is not a valid directory.", dir);
        return;
    }
    let subdirs = get_largest_subdirectories(dir, 10);
    if subdirs.is_empty() {
        println!("No subdirectories found or no files in directory.");
        return;
    }
    println!("\nTop 10 largest subdirectories:");
    for (i, (subdir, size)) in subdirs.iter().enumerate() {
        println!("{}. {} - {}", i + 1, subdir, human_readable_size(*size));
    }

    println!("\nEnter the number of a subdirectory to see its largest 10 files, or 'm' to return to the main menu:");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    let input = input.trim();
    if input.eq_ignore_ascii_case("m") {
        return;
    }
    if let Ok(idx) = input.parse::<usize>() {
        if idx >= 1 && idx <= subdirs.len() {
            let subdir = &subdirs[idx - 1].0;
            show_largest_files_in_dir(subdir, 10);
            // After showing, return to main menu
            return;
        } else {
            println!("Invalid selection. Please enter a valid number or 'm'.");
        }
    } else {
        println!("Invalid input. Please enter a number or 'm'.");
    }
}

//Returns a Vec of (subdir, total_size) for the largest immediate subdirectories
fn get_largest_subdirectories(dir: &str, top_n: usize) -> Vec<(String, u64)> {
    use std::collections::HashMap;
    use std::path::PathBuf;
    if !Path::new(dir).is_dir() {
        return Vec::new();
    }
    let mut dir_sizes: HashMap<String, u64> = HashMap::new();
    let base = Path::new(dir);
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            let path = entry.path();
            // Get immediate subdirectory under base
            if let Ok(rel) = path.strip_prefix(base) {
                if let Some(first) = rel.components().next() {
                    let mut subdir = PathBuf::from(dir);
                    subdir.push(first);
                    let subdir_str = subdir.display().to_string();
                    *dir_sizes.entry(subdir_str).or_insert(0) += size;
                }
            }
        }
    }
    let mut dir_sizes_vec: Vec<_> = dir_sizes.into_iter().collect();
    dir_sizes_vec.sort_by(|a, b| b.1.cmp(&a.1));
    dir_sizes_vec.into_iter().take(top_n).collect()
}

//Shows the largest files in a given directory
fn show_largest_files_in_dir(dir: &str, top_n: usize) {
    if !Path::new(dir).is_dir() {
        println!("{} is not a valid directory.", dir);
        return;
    }
    let mut largest_files = Vec::new();
    for entry in WalkDir::new(dir).min_depth(1).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            largest_files.push((entry.path().display().to_string(), size));
        }
    }
    //Also include files in subdirectories
    for entry in WalkDir::new(dir).min_depth(2).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            largest_files.push((entry.path().display().to_string(), size));
        }
    }
    largest_files.sort_by(|a, b| b.1.cmp(&a.1));
    println!("\nTop {} largest files in {}:", top_n, dir);
    for (file, size) in largest_files.iter().take(top_n) {
        println!("{} - {}", file, human_readable_size(*size));
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

//List available drives
fn list_available_drives() -> Vec<(char, String)> {
    let mut available_drives = Vec::new();
    for letter in b'A'..=b'Z' {
        let drive = format!("{}:/", letter as char);
        if std::path::Path::new(&drive).is_dir() {
            available_drives.push((letter as char, drive.clone()));
        }
    }
    available_drives
}

//Investigation loop for selecting subdirectories/files
fn investigation_loop(subdirs: &Vec<(String, u64)>) {
    loop {
        println!("\nEnter the number of a subdirectory to see its largest 10 files, or 'm' to return to the main menu:");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read input");
        let input = input.trim();
        if input.eq_ignore_ascii_case("m") {
            break;
        } else if let Ok(idx) = input.parse::<usize>() {
            if idx >= 1 && idx <= 10 && idx <= subdirs.len() {
                let subdir = &subdirs[idx - 1].0;
                show_largest_files_in_dir(subdir, 10);
            } else {
                println!("Invalid selection. Please enter a valid number or 'm'.");
            }
        } else {
            println!("Invalid input. Please enter a number or 'm'.");
        }
    }
}

//Normalize and deduplicate paths, helpful because for some reason, steam libraries were showing up funny.
fn normalize_and_dedup_paths(subdirs: &Vec<(String, u64)>) -> Vec<(String, u64)> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut deduped: Vec<(String, u64)> = Vec::new();
    for (subdir, size) in subdirs {
        let mut norm = subdir.replace('\\', "/");
        while norm.contains("//") { norm = norm.replace("//", "/"); }
        if norm.ends_with('/') { norm.pop(); }
        if seen.insert(norm.clone()) {
            deduped.push((norm, *size));
        }
    }
    deduped
}
}
