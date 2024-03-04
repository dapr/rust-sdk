mod github;

use core::panic;
use std::{error::Error, fs::File, io::BufReader, path::Path, process::exit};

use octocrab::models;

use github::GitHub;

// Defining the repo explicitly as the octocrab model for the event doesn't deserialize a
// owner/repo.
const OWNER: &str = "dapr";
const REPOSITORY: &str = "rust-sdk";

const GITHUB_TOKEN: &str = "GITHUB_TOKEN";

const GITHUB_EVENT_PATH: &str = "GITHUB_EVENT_PATH";
const GITHUB_EVENT_NAME: &str = "GITHUB_EVENT_NAME";

const ISSUE_COMMENT_EVENT_NAME: &str = "issue_comment";

fn get_payload<P: AsRef<Path>>(
    path: P,
) -> Result<models::events::payload::IssueCommentEventPayload, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance.
    let event = serde_json::from_reader(reader)?;

    // Return the event.
    Ok(event)
}

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let github_event_path: String =
        std::env::var(GITHUB_EVENT_PATH).expect("GITHUB_EVENT_PATH env missing");
    let github_event_name: String =
        std::env::var(GITHUB_EVENT_NAME).expect("GITHUB_EVENT_NAME env missing");
    let github_token: String = std::env::var(GITHUB_TOKEN).expect("GITHUB_TOKEN env missing");

    if github_event_name != ISSUE_COMMENT_EVENT_NAME {
        println!("Event is not an issue_comment, the app will now exit.");
        exit(exitcode::TEMPFAIL);
    }

    // deserialize event payload
    let event = get_payload(github_event_path).unwrap();

    // check the issue body
    if !event.clone().comment.body.unwrap().starts_with("/assign") {
        println!("Event does not start with /assign");
        exit(exitcode::TEMPFAIL);
    }

    let assignee: String = event.comment.user.login;

    let issue: u64 = event.issue.number;

    // spawn a client
    let github_client = GitHub::new_client(github_token);

    // assign the user
    match github_client
        .add_assignee(OWNER, REPOSITORY, issue, assignee.clone())
        .await
    {
        Ok(_) => {
            match github_client
                .create_comment(
                    OWNER,
                    REPOSITORY,
                    issue,
                    format!("🚀 issue assigned to you {assignee}"),
                )
                .await
            {
                Ok(_) => {
                    println!("Comment on assign successful")
                }
                Err(e) => {
                    panic!("Comment on assign failed: {:?}", e)
                }
            }
        }
        Err(e) => {
            panic!("Failed to assign issue: {:?}", e)
        }
    }

    Ok(())
}
