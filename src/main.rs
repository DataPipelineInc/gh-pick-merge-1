mod github_event;
mod helpers;

use chrono::prelude::*;
use helpers::github_get_commits_in_pr;

use helpers::*;

#[tokio::main]
async fn main() {
  git_setup();

  let github_event = get_event_action();

  let pr_number = github_event.number;

  let new_branch_name = create_new_branch_by_commits("develop".to_string(), pr_number)
    .await
    .expect("Create new branch by commit is failed");

  let pr_title = format!("chore: auto pick {}", pr_number);

  let body = format!("Auto pick merge by #{}", pr_number);

  let pull_request_id =
    github_open_pull_request(new_branch_name, "develop".to_string(), pr_title, body).await;
  github_pull_request_push_comment(pull_request_id, "test".to_string()).await;
}

fn generate_new_branch_name(to_branch: String) -> String {
  let timestamp: i64 = Utc::now().timestamp();

  format!("bot/auto-pick-{}-{:?}", to_branch, timestamp)
}

async fn create_new_branch_by_commits(to_branch: String, pr_number: i64) -> Option<String> {
  let origin_to_branch_name = format!("origin/{}", to_branch);

  let new_branch_name = generate_new_branch_name(to_branch);

  git(
    [
      "switch",
      "-c",
      new_branch_name.as_str(),
      origin_to_branch_name.as_str(),
    ]
    .to_vec(),
  );
  println!("New branch name:{}", new_branch_name);

  let not_matched_hash = pick_commits(pr_number).await;

  if not_matched_hash.len() > 0 {
    return None;
  }

  git(["push", "-u", "origin", new_branch_name.as_str()].to_vec());

  Some(new_branch_name)
}

async fn pick_commits(pr_number: i64) -> Vec<String> {
  let mut not_matched_hash = Vec::new();
  let commits = github_get_commits_in_pr(pr_number).await;

  for commit_hash in commits {
    let output = git(["cherry-pick", commit_hash.as_str()].to_vec());

    match output {
      Some(_output) => {
        println!("Pick success Commit hash: {:?}", commit_hash);
      }
      None => {
        not_matched_hash.push(commit_hash);
      }
    }
  }

  not_matched_hash
}
