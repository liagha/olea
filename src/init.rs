use std::process::{Command, Stdio};
use std::fs;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use std::ffi::CString;
use std::ptr;
use std::io::{self, Write, Read, BufRead, BufReader};
use std::collections::HashMap;

fn main() {
    println!("Starting custom init process...");

    // Set up signal handlers
    setup_signal_handlers();

    // Mount essential filesystems
    if let Err(err) = mount_virtual_filesystems() {
        println!("Warning: Error mounting filesystems: {}", err);
    }

    // Set up /dev nodes
    if let Err(err) = setup_dev() {
        println!("Warning: Error setting up /dev: {}", err);
    }

    // Try to find shell binary
    let shell_path = find_shell();
    println!("Using shell at: {}", shell_path);

    // List available binaries for debugging
    println!("Listing available binaries:");
    let paths_to_check = ["/bin", "/sbin", "/usr/bin", "/usr/sbin"];
    for path in paths_to_check.iter() {
        println!("Checking {}:", path);
        if Path::new(path).exists() {
            match Command::new("ls").arg("-la").arg(path).output() {
                Ok(output) => {
                    if let Ok(content) = String::from_utf8(output.stdout) {
                        println!("{}", content);
                    }
                },
                Err(e) => println!("Failed to list {}: {}", path, e),
            }
        } else {
            println!("{} directory doesn't exist", path);
        }
    }

    // Check for dynamic library dependencies with ldd if available
    println!("Checking dynamic library dependencies for shell...");
    match Command::new("ldd").arg(&shell_path).output() {
        Ok(output) => {
            if let Ok(content) = String::from_utf8(output.stdout) {
                println!("Dynamic dependencies:\n{}", content);
            } else {
                println!("Failed to decode ldd output");
            }
        },
        Err(e) => println!("Failed to run ldd: {} - ldd might not be available", e),
    }

    // Try to examine the binary type
    println!("Checking binary type with file command...");
    match Command::new("file").arg(&shell_path).output() {
        Ok(output) => {
            if let Ok(content) = String::from_utf8(output.stdout) {
                println!("Binary type: {}", content.trim());
            } else {
                println!("Failed to decode file command output");
            }
        },
        Err(e) => println!("Failed to run file command: {} - file might not be available", e),
    }

    // Check if /lib exists and list its contents
    println!("Checking library directories:");
    for lib_dir in ["/lib", "/lib64", "/usr/lib", "/distro/lib"] {
        if Path::new(lib_dir).exists() {
            println!("  {} exists, listing contents:", lib_dir);
            match Command::new("ls").arg("-la").arg(lib_dir).output() {
                Ok(output) => {
                    if let Ok(content) = String::from_utf8(output.stdout) {
                        println!("{}", content);
                    }
                },
                Err(e) => println!("    Failed to list {}: {}", lib_dir, e),
            }
        } else {
            println!("  {} does not exist", lib_dir);
        }
    }

    // Try running a minimal C program directly
    println!("\nTrying to use a simple command instead:");
    for simple_cmd in ["echo", "cat", "ls"] {
        println!("Attempting to run {}...", simple_cmd);
        match Command::new(simple_cmd).arg("Hello from init").status() {
            Ok(status) => {
                println!("  {} executed successfully with status: {}", simple_cmd, status);
                break; // If one works, stop trying
            },
            Err(e) => println!("  Failed to execute {}: {}", simple_cmd, e),
        }
    }

    // Try a static binary if available or try to use busybox
    println!("\nLooking for static shells or busybox...");
    for static_shell in ["/bin/busybox", "/sbin/busybox", "/bin/sh.static", "/bin/bash.static"] {
        if Path::new(static_shell).exists() {
            println!("Found possible static binary at {}, attempting to execute...", static_shell);
            let args = if static_shell.contains("busybox") { vec!["sh"] } else { vec![] };

            let mut cmd = Command::new(static_shell);
            for arg in args {
                cmd.arg(arg);
            }

            match cmd.status() {
                Ok(status) => println!("Static shell exited with status: {}", status),
                Err(e) => println!("Failed to execute static shell: {}", e),
            }
            break;
        }
    }

    // As a last resort, try the original shell with more information
    println!("\nAttempting to start original shell at {}...", shell_path);
    match Command::new(&shell_path).status() {
        Ok(status) => println!("Shell exited with status: {}", status),
        Err(e) => {
            println!("Failed to execute shell: {} - continuing anyway", e);
            println!("Checking file permissions and existence:");
            if Path::new(&shell_path).exists() {
                match fs::metadata(&shell_path) {
                    Ok(meta) => println!("  File exists with permissions: {:o}", meta.permissions().mode()),
                    Err(e) => println!("  Failed to get metadata: {}", e),
                }

                // Check if it's a symlink and resolve it
                match fs::read_link(&shell_path) {
                    Ok(target) => {
                        println!("  File is a symlink pointing to: {:?}", target);
                        println!("  Checking if target exists...");
                        if Path::new(&target).exists() {
                            println!("  Target exists");
                        } else {
                            println!("  Target does not exist!");
                        }
                    },
                    Err(_) => println!("  File is not a symlink"),
                }
            } else {
                println!("  Shell file does not exist at {}", shell_path);
            }
        }
    }

    // Create our own minimal shell if all else fails
    println!("\nCreating a minimal emergency shell...");
    emergency_shell();

    // Loop to prevent init from exiting and handle child processes
    println!("Init process looping...");
    loop {
        // Wait for any child process to change state
        let mut status: libc::c_int = 0;
        let wait_result = unsafe { libc::waitpid(-1, &mut status, libc::WNOHANG) };

        match wait_result {
            -1 => {
                let err = std::io::Error::last_os_error();
                if err.raw_os_error() != Some(libc::ECHILD) {
                    println!("waitpid() error: {}", err);
                }
            },
            0 => {}, // No status change
            pid => println!("Process {} status changed: {}", pid, status),
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn find_shell() -> String {
    // Try to find a valid shell binary
    let possible_shells = [
        "/bin/sh",
        "/bin/bash",
        "/sbin/sh",
        "/usr/bin/sh",
        "/usr/bin/bash",
        "/distro/initramfs/bin/bash", // Based on the symlink we saw
        "/distro/bin/bash",
        "/distro/bin/sh",
        "sh", // Try using PATH
        "bash"
    ];

    // First check for BusyBox which might work if it's statically linked
    let busybox_paths = ["/bin/busybox", "/sbin/busybox", "/usr/bin/busybox"];
    for busybox in busybox_paths.iter() {
        if Path::new(busybox).exists() {
            println!("Found BusyBox at {}, which might provide a shell", busybox);
            return busybox.to_string(); // We'll handle adding "sh" argument when executing
        }
    }

    // Then try regular shells
    for shell in possible_shells.iter() {
        if Path::new(shell).exists() || which(shell).is_some() {
            // If it's a symlink, try to resolve final target
            match fs::read_link(shell) {
                Ok(target) => {
                    println!("Found shell symlink at {} -> {:?}", shell, target);
                    let target_str = target.to_string_lossy();
                    if target_str.starts_with("/") {
                        // Absolute path
                        if Path::new(&target).exists() {
                            return target.to_string_lossy().into_owned();
                        }
                    } else {
                        // Relative path
                        let shell_path = Path::new(shell);
                        if let Some(parent) = shell_path.parent() {
                            let full_target_path = parent.join(&target);
                            if full_target_path.exists() {
                                return full_target_path.to_string_lossy().into_owned();
                            }
                        }
                    }
                    // If target doesn't exist, still return the original shell path
                    return shell.to_string();
                },
                Err(_) => {
                    // Not a symlink, use directly
                    return shell.to_string();
                }
            }
        }
    }

    // Default to /bin/sh even if not found - will generate expected error
    "/bin/sh".to_string()
}

fn which(program: &str) -> Option<String> {
    if program.contains('/') {
        return if Path::new(program).exists() {
            Some(program.to_string())
        } else {
            None
        };
    }

    match std::env::var_os("PATH") {
        Some(paths) => {
            for path in std::env::split_paths(&paths) {
                let full_path = path.join(program);
                if full_path.exists() {
                    return full_path.to_str().map(String::from);
                }
            }
            None
        }
        None => None,
    }
}

fn setup_signal_handlers() {
    unsafe {
        // Don't ignore SIGCHLD, we want to handle it properly
        libc::signal(libc::SIGTERM, libc::SIG_IGN);

        // Using SIG_DFL instead of SIG_IGN for SIGCHLD to allow proper waitpid functionality
        libc::signal(libc::SIGCHLD, libc::SIG_DFL);
    }
}

fn mount_virtual_filesystems() -> Result<(), String> {
    println!("Mounting virtual filesystems...");

    for dir in &["/proc", "/sys", "/dev", "/tmp"] {
        if !Path::new(dir).exists() {
            fs::create_dir_all(dir)
                .map_err(|e| format!("Failed to create {}: {}", dir, e))?;
        }
    }

    let mount_points = [
        ("proc", "/proc", "proc"),
        ("sysfs", "/sys", "sysfs"),
        ("devtmpfs", "/dev", "devtmpfs"),
        ("tmpfs", "/tmp", "tmpfs"),
    ];

    for (source, target, fs_type) in mount_points.iter() {
        let source_c = CString::new(*source).map_err(|e| e.to_string())?;
        let target_c = CString::new(*target).map_err(|e| e.to_string())?;
        let fs_type_c = CString::new(*fs_type).map_err(|e| e.to_string())?;
        let result = unsafe {
            libc::mount(
                source_c.as_ptr(),
                target_c.as_ptr(),
                fs_type_c.as_ptr(),
                0,
                ptr::null(),
            )
        };
        if result != 0 {
            let err = std::io::Error::last_os_error();
            println!("Failed to mount {}: {}", target, err);
        } else {
            println!("Successfully mounted {}", target);
        }
    }

    Ok(())
}

fn setup_dev() -> Result<(), String> {
    println!("Setting up /dev...");

    let devices = [
        ("/dev/console", 0o620),
        ("/dev/null", 0o666),
        ("/dev/zero", 0o666),
        ("/dev/tty", 0o666),
    ];

    for (device, mode) in devices.iter() {
        if Path::new(device).exists() {
            match fs::metadata(device) {
                Ok(meta) => {
                    let mut perms = meta.permissions();
                    perms.set_mode(*mode);
                    if let Err(e) = fs::set_permissions(device, perms) {
                        println!("Warning: Failed to set permissions on {}: {}", device, e);
                    } else {
                        println!("Successfully set permissions on {}", device);
                    }
                },
                Err(e) => {
                    println!("Warning: Failed to get metadata for {}: {}", device, e);
                }
            }
        } else {
            println!("Warning: {} does not exist", device);
        }
    }

    Ok(())
}

// Basic emergency shell implementation
fn emergency_shell() {
    println!("\n--------------------------------------");
    println!("Emergency Init Shell");
    println!("Type 'help' for available commands");
    println!("--------------------------------------\n");

    let mut running = true;

    while running {
        print!("init> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if let Err(e) = io::stdin().read_line(&mut input) {
            println!("Error reading input: {}", e);
            continue;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0];
        let args = &parts[1..];

        match command {
            "exit" | "quit" => {
                println!("Exiting emergency shell");
                running = false;
            },
            "help" => {
                println!("Available commands:");
                println!("  help           - Show this help message");
                println!("  exit/quit      - Exit the emergency shell");
                println!("  ls [dir]       - List directory contents");
                println!("  cd [dir]       - Change current directory");
                println!("  cat [file]     - Display file contents");
                println!("  mkdir [dir]    - Create directory");
                println!("  rm [file]      - Remove file");
                println!("  touch [file]   - Create empty file");
                println!("  env            - Show environment variables");
                println!("  mount          - Show mounted filesystems");
                println!("  run [cmd]      - Run a command using Command");
                println!("  mounts         - List mount points");
                println!("  ldd [path]     - Show dynamic dependencies");
                println!("  file [path]    - Show file type");
            },
            "ls" => {
                let dir = if args.is_empty() { "." } else { args[0] };
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            print!("{}", entry.file_name().to_string_lossy());
                            if let Ok(metadata) = entry.metadata() {
                                if metadata.is_dir() {
                                    print!("/");
                                } else if metadata.permissions().mode() & 0o111 != 0 {
                                    print!("*");
                                }
                            }
                            println!();
                        }
                    }
                } else {
                    println!("Failed to read directory {}", dir);
                }
            },
            "cd" => {
                let dir = if args.is_empty() { "/" } else { args[0] };
                if let Err(e) = std::env::set_current_dir(dir) {
                    println!("Failed to change directory: {}", e);
                } else {
                    println!("Changed to {}", dir);
                }
            },
            "cat" => {
                if args.is_empty() {
                    println!("Usage: cat [file]");
                } else {
                    match fs::read_to_string(args[0]) {
                        Ok(content) => println!("{}", content),
                        Err(e) => println!("Failed to read file: {}", e),
                    }
                }
            },
            "mkdir" => {
                if args.is_empty() {
                    println!("Usage: mkdir [dir]");
                } else {
                    if let Err(e) = fs::create_dir_all(args[0]) {
                        println!("Failed to create directory: {}", e);
                    } else {
                        println!("Directory created");
                    }
                }
            },
            "rm" => {
                if args.is_empty() {
                    println!("Usage: rm [file]");
                } else {
                    if let Err(e) = fs::remove_file(args[0]) {
                        println!("Failed to remove file: {}", e);
                    } else {
                        println!("File removed");
                    }
                }
            },
            "touch" => {
                if args.is_empty() {
                    println!("Usage: touch [file]");
                } else {
                    match fs::File::create(args[0]) {
                        Ok(_) => println!("File created"),
                        Err(e) => println!("Failed to create file: {}", e),
                    }
                }
            },
            "env" => {
                for (key, value) in std::env::vars() {
                    println!("{}={}", key, value);
                }
            },
            "mount" => {
                if let Ok(content) = fs::read_to_string("/proc/mounts") {
                    println!("{}", content);
                } else {
                    println!("Failed to read mounts");
                }
            },
            "run" => {
                if args.is_empty() {
                    println!("Usage: run [cmd] [args...]");
                } else {
                    let mut cmd = Command::new(args[0]);
                    if args.len() > 1 {
                        cmd.args(&args[1..]);
                    }
                    match cmd.status() {
                        Ok(status) => println!("Command exited with status: {}", status),
                        Err(e) => println!("Failed to execute command: {}", e),
                    }
                }
            },
            "mounts" => {
                let mount_points = [
                    ("/proc", "Process information"),
                    ("/sys", "System information"),
                    ("/dev", "Device files"),
                    ("/tmp", "Temporary files"),
                ];

                println!("Mount points:");
                for (mount, desc) in &mount_points {
                    if Path::new(mount).exists() {
                        println!("  {} - {} [EXISTS]", mount, desc);
                    } else {
                        println!("  {} - {} [MISSING]", mount, desc);
                    }
                }
            },
            "ldd" => {
                if args.is_empty() {
                    println!("Usage: ldd [path]");
                } else {
                    match Command::new("ldd").arg(args[0]).output() {
                        Ok(output) => {
                            if let Ok(content) = String::from_utf8(output.stdout) {
                                println!("{}", content);
                            } else {
                                println!("Failed to decode ldd output");
                            }
                        },
                        Err(e) => println!("Failed to run ldd: {}", e),
                    }
                }
            },
            "file" => {
                if args.is_empty() {
                    println!("Usage: file [path]");
                } else {
                    match Command::new("file").arg(args[0]).output() {
                        Ok(output) => {
                            if let Ok(content) = String::from_utf8(output.stdout) {
                                println!("{}", content);
                            } else {
                                println!("Failed to decode file command output");
                            }
                        },
                        Err(e) => println!("Failed to run file command: {}", e),
                    }
                }
            },
            _ => {
                println!("Unknown command: {}", command);
                println!("Type 'help' for available commands");
            }
        }
    }
}