mod branch;
mod github_env;
mod github_event;
mod helpers;
mod pull_request;
mod types;

use crate::branch::*;
use crate::github_event::*;
use crate::helpers::git;
use crate::helpers::git_setup;
use crate::helpers::match_pick_merge_labels;
use types::*;

#[tokio::main]
async fn main() {
  let github_event = get_event_action().await;
  let matched_labels = match_pick_merge_labels(github_event.pull_request.labels);

  if matched_labels.len() <= 0 {
    return;
  }

  git_setup();

  for label in matched_labels {
    println!("dest branch: {}", label);

    let head_branch = github_event.pull_request.head._ref.as_str();
    let dest_branch = label.split("/").last().expect("Not match dest branch");

    git(["fetch", "origin", head_branch].to_vec());
    git(["fetch", "origin", dest_branch].to_vec());

    pick_pr_to_dest_branch(
      github_event.number,
      &github_event.pull_request.title,
      dest_branch.to_string(),
    )
    .await;
  }
}
