use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        process::exit(0);
    }

    let command = &args[1];

    match command.as_str() {
        "list" => {
            if let Err(e) = list_hosts() {
                eprintln!("Error listing hosts: {}", e);
                process::exit(1);
            }
        }
        "help" | "--help" | "-h" => {
            print_help();
        }
        _ => {
            // Assume it's an IP address to remove
            if let Err(e) = remove_host(&command) {
                eprintln!("Error removing host: {}", e);
                process::exit(1);
            }
        }
    }
}

fn print_help() {
    println!("ssr - SSH known_hosts manager");
    println!();
    println!("USAGE:");
    println!("    ssr list              List all entries in known_hosts");
    println!("    ssr <ip_or_hostname>  Remove entries matching the given IP or hostname");
    println!("    ssr help              Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    ssr list");
    println!("    ssr 192.168.1.100");
    println!("    ssr example.com");
}

fn get_known_hosts_path() -> io::Result<PathBuf> {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not find home directory"))?;
    
    let mut path = PathBuf::from(home);
    path.push(".ssh");
    path.push("known_hosts");
    
    Ok(path)
}

fn list_hosts() -> io::Result<()> {
    let path = get_known_hosts_path()?;
    
    if !path.exists() {
        println!("No known_hosts file found at: {}", path.display());
        return Ok(());
    }

    let contents = fs::read_to_string(&path)?;
    
    if contents.trim().is_empty() {
        println!("The known_hosts file is empty.");
        return Ok(());
    }

    println!("Entries in {}", path.display());
    println!("{}", "=".repeat(60));
    
    use std::collections::HashMap;
    let mut host_counts: HashMap<String, usize> = HashMap::new();
    let mut seen_order: Vec<String> = Vec::new();
    
    for line in contents.lines() {
        if line.trim().is_empty() || line.trim().starts_with('#') {
            continue;
        }
        
        let parts: Vec<&str> = line.split_whitespace().collect();
        if !parts.is_empty() {
            let host = parts[0].to_string();
            
            if !host_counts.contains_key(&host) {
                seen_order.push(host.clone());
            }
            
            *host_counts.entry(host).or_insert(0) += 1;
        }
    }
    
    for (i, host) in seen_order.iter().enumerate() {
        let count = host_counts.get(host).unwrap();
        if *count > 1 {
            println!("{:4}. {} ({})", i + 1, host, count);
        } else {
            println!("{:4}. {}", i + 1, host);
        }
    }
    
    Ok(())
}

fn remove_host(target: &str) -> io::Result<()> {
    let path = get_known_hosts_path()?;
    
    if !path.exists() {
        println!("No known_hosts file found at: {}", path.display());
        return Ok(());
    }

    let contents = fs::read_to_string(&path)?;
    let mut new_contents = Vec::new();
    let mut removed_count = 0;

    for line in contents.lines() {
        if line.trim().is_empty() || line.trim().starts_with('#') {
            new_contents.push(line.to_string());
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if !parts.is_empty() {
            let host_part = parts[0];
            
            // Check if the target matches any part of the host field
            // The host field can be: hostname, [hostname]:port, or multiple comma-separated
            let matches = host_part.split(',').any(|h| {
                let clean_host = h.trim_start_matches('[').split(']').next().unwrap_or(h);
                clean_host == target || h.contains(target)
            });

            if matches {
                removed_count += 1;
                println!("Removing entry: {}", host_part);
            } else {
                new_contents.push(line.to_string());
            }
        } else {
            new_contents.push(line.to_string());
        }
    }

    if removed_count == 0 {
        println!("No entries found matching '{}'", target);
        return Ok(());
    }

    // Write the new contents back to the file
    let mut file = fs::File::create(&path)?;
    for line in new_contents {
        writeln!(file, "{}", line)?;
    }

    println!("Successfully removed {} entry/entries for '{}'", removed_count, target);
    
    Ok(())
}