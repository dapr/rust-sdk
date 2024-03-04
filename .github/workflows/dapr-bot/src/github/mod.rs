use octocrab::Octocrab;

pub mod issues;

pub struct GitHub {
    client: Octocrab,
}

impl GitHub {
    pub fn new_client(token: String) -> GitHub {
        match Octocrab::builder().personal_token(token).build() {
            Ok(client) => GitHub { client },
            Err(e) => {
                panic!("Unable to create client: {:?}", e)
            }
        }
    }
}
