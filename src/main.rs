// git fetch --prune && git branch -r | awk "{print \$1}" | egrep -v -f /dev/fd/0 <(`git branch -vv` | grep origin) | awk "{print \$1}" | xargs git branch -d

use clap::Parser;
use std::process::Command;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[arg(short, long)]
    all: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Arguments::parse();
    println!("NOTE: If this is run in the built-in VS Code terminal with Git Bash. It will not work. Run it in a normal terminal or in CMD, PowerShell, etc.");
    println!("Fetching remote branches...");
    let status = Command::new("git")
        .args(["fetch", "--prune"])
        .status()
        .expect("failed to execute git fetch --prune");

    if status.success() {
        let output = Command::new("git")
            .args(["branch", "-v"])
            .output()
            .expect("Failed to execute git branch -v");
        let output = String::from_utf8(output.stdout)?;
        let output = output.trim();
        if output.is_empty() {
            println!("No remote branches found: git branch -v");
            return Ok(());
        }

        let branches = get_branches_from_output(output);

        let failed_deletions = delete_branches(branches)?;

        if !failed_deletions.is_empty() && !args.all {
            force_deletion_if_approved(failed_deletions)?;
        } else if !failed_deletions.is_empty() && args.all {
            force_delete_branches(failed_deletions)?;
        }
    } else {
        return Err(anyhow::anyhow!(
            "git fetch --prune failed with exit code {:?}",
            status.code(),
        ));
    }
    Ok(())
}

fn delete_branches(branches: Vec<String>) -> anyhow::Result<Vec<String>> {
    let mut failed_deletions = Vec::new();
    for branch in branches {
        let status = Command::new("git")
            .args(["branch", "-d", &branch])
            .output()
            .expect("failed to execute git branch -d");
        if !status.status.success() {
            failed_deletions.push(branch);
        } else {
            println!("Deleted branch: {}", branch);
        }
    }
    Ok(failed_deletions)
}

fn force_deletion_if_approved(failed_deletions: Vec<String>) -> anyhow::Result<()> {
    let question = requestty::Question::multi_select("deleting")
        .message("Select the branches that you want to forcibly delete")
        .choices(failed_deletions)
        .build();
    let result = requestty::prompt_one(question)?;
    let result = result
        .try_into_list_items()
        .expect("Error getting results")
        .into_iter()
        .map(|r| r.text)
        .collect();
    force_delete_branches(result)
}

fn force_delete_branches(branches: Vec<String>) -> anyhow::Result<()> {
    for r in branches {
        println!("Deleting branch: {}", r);
        let output = Command::new("git")
            .args(["branch", "-D", &r])
            .output()
            .expect("failed to execute git branch -D");
        if !output.status.success() {
            println!("Failed to delete branch: {}", r);
        }
    }
    Ok(())
}

fn get_branches_from_output(output: &str) -> Vec<String> {
    let branches: Vec<&str> = output.trim().split('\n').collect();
    let branches: Vec<&&str> = branches
        .iter()
        .filter(|b| b.contains("[gone]") && !b.contains('*'))
        .collect();
    let branches: Vec<String> = branches
        .iter()
        .map(|b| {
            let v: Vec<&str> = b.trim().split(' ').collect();
            v[0].trim().to_string()
        })
        .collect();
    branches
}
