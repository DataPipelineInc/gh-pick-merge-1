use crate::helpers::*;
use crate::pull_request::github_get_commits_in_pr;
use crate::pull_request::github_open_pull_request;
use crate::pull_request::github_pull_request_push_comment;
use crate::types::*;

pub async fn pick_pr_to_dest_branch(pr_number: i64, pr_title: &String, dest_branch: String) {
  println!("Start job pick to: {}", dest_branch);

  let create_branch_result = create_new_branch_by_commits(dest_branch.clone(), pr_number).await;
  let comment = format!("Auto pick merge by #{}", pr_number);

  let pull_request_id = github_open_pull_request(
    create_branch_result.new_branch_name,
    dest_branch,
    pr_title.to_string(),
    comment,
  )
  .await;
  
  if create_branch_result.not_matched_hash.len() > 0 {
    github_pull_request_push_comment(
      pull_request_id,
      generate_pull_request_comment(create_branch_result.not_matched_hash),
    )
    .await;
  }

  println!("End job");
}

async fn create_new_branch_by_commits(to_branch: String, pr_number: i64) -> CreateNewBranchResult {
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

  git(["push", "-u", "origin", new_branch_name.as_str()].to_vec());

  CreateNewBranchResult::new(new_branch_name, not_matched_hash)
}

async fn pick_commits(pr_number: i64) -> Vec<String> {
  let mut matched_hash = Vec::new();
  let mut not_matched_hash = Vec::new();
  let commits = github_get_commits_in_pr(pr_number).await;

  for commit_hash in commits {
    let output = git(["cherry-pick", commit_hash.as_str()].to_vec());

    match output {
      Some(output) => {
        matched_hash.push(commit_hash.clone());
        println!(
          "Pick success Commit hash: {:?}, output: {:?}",
          commit_hash, output
        );
      }
      None => {
        git(["cherry-pick", "--abort"].to_vec());
        not_matched_hash.push(commit_hash);
        println!("Pick error abort",);
      }
    }
  }

  if matched_hash.len() == 0 {
    let output = git(["commit", "--allow-empty", "-m", "empty: no matched commit"].to_vec());
    println!("Commit allow empty output: {:?}", output);
  }

  not_matched_hash
}
