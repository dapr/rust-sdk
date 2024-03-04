use super::GitHub;

impl GitHub {
    pub async fn create_comment(
        &self,
        owner: &str,
        repo: &str,
        number: u64,
        comment: String,
    ) -> std::result::Result<octocrab::models::issues::Comment, octocrab::Error> {
        self.client
            .issues(owner, repo)
            .create_comment(number, comment)
            .await
    }
    pub async fn add_assignee(
        &self,
        owner: &str,
        repo: &str,
        number: u64,
        assignee: String,
    ) -> std::result::Result<octocrab::models::issues::Issue, octocrab::Error> {
        self.client
            .issues(owner, repo)
            .add_assignees(number, &[&assignee])
            .await
    }
}
