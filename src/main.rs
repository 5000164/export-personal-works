use serde::{Deserialize, Serialize};
use std::env;

fn main() {
    let repositories = get_repositories()
        .unwrap()
        .into_iter()
        .filter(|r| r.fork == false)
        .collect::<Vec<_>>();

    let mut repository_statistics = repositories
        .iter()
        .map(|r| calculate_repository_statistic(&r.name, &r.html_url))
        .collect::<Vec<_>>();
    repository_statistics.sort_by(|a, b| b.last_commit_date.cmp(&a.last_commit_date));
    let serialized = serde_json::to_string(&repository_statistics).unwrap();
    println!("{}", serialized);
}

fn calculate_repository_statistic(
    repository_name: &str,
    repository_url: &str,
) -> RepositoryStatistic {
    let meta_commits = get_meta_commits(repository_name, 1).unwrap();

    let repository_statistic = RepositoryStatistic {
        name: repository_name.to_string(),
        url: repository_url.to_string(),
        first_commit_date: meta_commits
            .clone()
            .into_iter()
            .last()
            .unwrap()
            .commit
            .author
            .date,
        last_commit_date: meta_commits
            .clone()
            .into_iter()
            .next()
            .unwrap()
            .commit
            .author
            .date,
        commit_count: meta_commits.len(),
    };

    if meta_commits.len() < 100 {
        repository_statistic
    } else {
        calculate_recursive(repository_statistic, 2)
    }
}

fn calculate_recursive(
    repository_statistic: RepositoryStatistic,
    page: usize,
) -> RepositoryStatistic {
    let meta_commits = get_meta_commits(&repository_statistic.name, page).unwrap();

    let updated_repository_statistic = RepositoryStatistic {
        name: repository_statistic.name,
        url: repository_statistic.url,
        first_commit_date: meta_commits
            .clone()
            .into_iter()
            .last()
            .unwrap()
            .commit
            .author
            .date,
        last_commit_date: repository_statistic.last_commit_date,
        commit_count: repository_statistic.commit_count + meta_commits.len(),
    };

    if meta_commits.len() < 100 {
        updated_repository_statistic
    } else {
        calculate_recursive(updated_repository_statistic, page + 1)
    }
}

fn get_repositories() -> Result<Vec<Repository>, ureq::Error> {
    let body: String = ureq::get("https://api.github.com/users/5000164/repos")
        .set(
            "Authorization",
            &format!("token {}", env::var("TOKEN").unwrap()),
        )
        .query("per_page", "100")
        .query("page", "1")
        .call()?
        .into_string()?;
    Ok(serde_json::from_str(&body).unwrap())
}

fn get_meta_commits(repository_name: &str, page: usize) -> Result<Vec<MetaCommit>, ureq::Error> {
    let body: String = ureq::get(&format!(
        "https://api.github.com/repos/5000164/{}/commits",
        repository_name
    ))
    .set(
        "Authorization",
        &format!("token {}", env::var("TOKEN").unwrap()),
    )
    .query("per_page", "100")
    .query("page", &page.to_string())
    .call()?
    .into_string()?;
    Ok(serde_json::from_str(&body).unwrap())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RepositoryStatistic {
    name: String,
    url: String,
    first_commit_date: String,
    last_commit_date: String,
    commit_count: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Repository {
    name: String,
    html_url: String,
    fork: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MetaCommit {
    commit: Commit,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Commit {
    author: Author,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Author {
    date: String,
}
