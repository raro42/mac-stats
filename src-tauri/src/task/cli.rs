//! CLI for task operations. Invoked from main when `mac_stats --task <subcommand>` is used.

use clap::Subcommand;

/// Task CLI subcommands. Parsed by main and passed to run().
#[derive(Subcommand, Debug)]
pub enum TaskCmd {
    /// Create a new task
    Add {
        topic: String,
        id: String,
        #[arg(default_value = "")]
        content: String,
    },
    /// List tasks (open + WIP by default; use --all for all statuses)
    List {
        #[arg(long)]
        all: bool,
    },
    /// Show one task (status and full content)
    Show {
        id: String,
    },
    /// Get or set task status
    Status {
        id: String,
        /// If provided, set status to this value (open|wip|finished|unsuccessful)
        status: Option<String>,
    },
    /// Remove a task (deletes all status files for that task)
    Remove {
        id: String,
    },
    /// Assign task to an agent (scheduler|discord|cpu|default)
    Assign {
        id: String,
        agent: String,
    },
    /// Append feedback to a task
    Append {
        id: String,
        content: String,
    },
}

/// Run the task CLI subcommand. Prints to stdout/stderr. Returns Ok(()) on success, Err(exit_code) on failure.
pub fn run(cmd: TaskCmd) -> Result<(), i32> {
    match cmd {
        TaskCmd::Add { topic, id, content } => {
            match crate::task::create_task(&topic, &id, &content, None) {
                Ok(path) => {
                    println!("Created: {}", path.display());
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    Err(1)
                }
            }
        }
        TaskCmd::List { all } => {
            let result = if all {
                crate::task::format_list_all_tasks()
            } else {
                crate::task::format_list_open_and_wip_tasks()
            };
            match result {
                Ok(s) => {
                    println!("{}", s);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    Err(1)
                }
            }
        }
        TaskCmd::Show { id } => {
            let path = match crate::task::resolve_task_path(&id) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return Err(1);
                }
            };
            let (status, assignee, content) = match crate::task::show_task_content(&path) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return Err(1);
                }
            };
            println!("Status: {}", status);
            println!("Assigned: {}", assignee);
            println!("Path: {}", path.display());
            println!("---");
            println!("{}", content);
            Ok(())
        }
        TaskCmd::Status { id, status } => {
            let path = match crate::task::resolve_task_path(&id) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return Err(1);
                }
            };
            match status {
                None => {
                    let s = crate::task::status_from_path(&path)
                        .unwrap_or_else(|| "?".to_string());
                    println!("{}", s);
                    Ok(())
                }
                Some(new_status) => {
                    let new_status = new_status.to_lowercase();
                    if !["open", "wip", "finished", "unsuccessful"].contains(&new_status.as_str()) {
                        eprintln!("Invalid status. Use: open, wip, finished, unsuccessful");
                        return Err(1);
                    }
                    match crate::task::set_task_status(&path, &new_status) {
                        Ok(p) => {
                            println!("Status set to {}: {}", new_status, p.display());
                            Ok(())
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            Err(1)
                        }
                    }
                }
            }
        }
        TaskCmd::Remove { id } => {
            match crate::task::delete_task(&id) {
                Ok(n) => {
                    println!("Removed {} file(s) for task {}", n, id);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    Err(1)
                }
            }
        }
        TaskCmd::Assign { id, agent } => {
            let path = match crate::task::resolve_task_path(&id) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return Err(1);
                }
            };
            match crate::task::set_assignee(&path, &agent) {
                Ok(()) => {
                    if let Err(e) = crate::task::append_to_task(&path, &format!("Reassigned to {}.", agent)) {
                        eprintln!("Warning: append note failed: {}", e);
                    }
                    println!("Assigned task {} to {}", id, agent);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    Err(1)
                }
            }
        }
        TaskCmd::Append { id, content } => {
            let path = match crate::task::resolve_task_path(&id) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return Err(1);
                }
            };
            match crate::task::append_to_task(&path, &content) {
                Ok(()) => {
                    println!("Appended to task {}", id);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    Err(1)
                }
            }
        }
    }
}
