use crate::hub::RepoLocator;
use failure::Error;
use std::fs;
use crate::hub::CommitLocator;
use git2::Oid;
use git2::Repository;
use git2::ResetType;

pub struct LocalRepo {
    path: String,
    default_branch: String,
    git: Repository,
}

impl LocalRepo {
    pub fn new(user: &str, token: &str, locator: &RepoLocator, branch: &str, context: &str) -> Result<Self, Error> {
        let url = format!("https://{}:{}@github.com/{}/{}.git", &user, &token, &locator.owner, &locator.repo);
        let path = format!("/tmp/crane/{}/{}/{}", &locator.owner, &locator.repo, &context);
        fs::remove_dir_all(&path).unwrap_or(());
        fs::create_dir_all(&path)?;
        let repo = LocalRepo {
            path: path.clone(),
            default_branch: branch.to_string(),
            git: Repository::clone(&url, &path)?,
        };
        Ok(repo)
    }

    pub fn reset_to(&mut self, commit: &CommitLocator) -> Result<(), Error> {
        self.git.find_remote("origin")?
            .fetch(&[&self.default_branch], None, None)?;
        let git_commit = self.git.find_commit(Oid::from_str(&commit.sha)?)?;
        self.git.reset(&git_commit.as_object(), ResetType::Hard, None)?;
        Ok(())
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}
