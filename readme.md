#SSR

A simple tool written in Rust to remove entries for a certain ip address from the current users .ssh/known_hosts file.

Compile with ```cargo build --release```

Install with ```cargo install --path .```

USAGE:
    ssr list              List all entries in known_hosts
    ssr <ip_or_hostname>  Remove entries matching the given IP or hostname
    ssr help              Show this help message

EXAMPLES:
    ssr list
    ssr 192.168.1.100
    ssr example.com
