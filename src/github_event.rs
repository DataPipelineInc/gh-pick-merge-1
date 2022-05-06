use crate::github_env::get_github_env;
use crate::github_env::is_travis;
use crate::helpers::get_github_api;
use crate::helpers::github_api_event_repo_url;
use std::{env, fs};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde()]
pub struct GithubActionPullRequestBranch {
  #[serde(rename(deserialize = "ref"))]
  pub _ref: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubActionPullRequestLabel {
  id: i64,
  pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubActionPullRequest {
  pub number: i64,
  pub title: String,
  pub base: GithubActionPullRequestBranch,
  pub head: GithubActionPullRequestBranch,
  pub labels: Vec<GithubActionPullRequestLabel>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubEventAction {
  action: String,
  pub number: i64,
  pub pull_request: GithubActionPullRequest,
}

// ------ Response ------

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubGetCommitResponseItem {
  pub sha: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubCreatePullRequestResponse {
  pub number: i64,
}

// ------ Impl ------

pub async fn get_event_action() -> GithubEventAction {
  if is_travis() == true {
    return get_event_action_by_api().await;
  }

  get_event_action_by_gh_action()
}

// https://docs.github.com/en/developers/webhooks-and-events/events/github-event-types#pullrequestevent
pub fn get_event_action_by_gh_action() -> GithubEventAction {
  let github_event_path = env::var_os("GITHUB_EVENT_PATH").unwrap();
  let github_event_string = fs::read_to_string(github_event_path)
    .expect("Github action read event file to string is failed");

  serde_json::from_str::<GithubEventAction>(&github_event_string)
    .expect("Convert to github action event is failed")
}

pub async fn get_event_action_by_api() -> GithubEventAction {
  let env = get_github_env();
  let repo_url = github_api_event_repo_url();
  let url = format!("{}/pulls/{}", repo_url, env.pull_request_number);

  get_github_api(url)
    .await
    .json::<GithubEventAction>()
    .await
    .expect("Failed into json by pull request event")
}
