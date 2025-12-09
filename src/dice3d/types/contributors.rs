//! Contributors data types
//!
//! This module contains types for loading and displaying GitHub contributors.
//! Contributors are fetched directly from the GitHub API at runtime.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// GitHub API response for a contributor
#[derive(Debug, Clone, Deserialize)]
struct GitHubContributor {
    login: String,
    avatar_url: String,
    html_url: String,
    contributions: u32,
}

/// GitHub API response for user details
#[derive(Debug, Clone, Deserialize)]
struct GitHubUser {
    name: Option<String>,
}

/// A single contributor entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contributor {
    /// GitHub username
    pub login: String,
    /// Display name (may be null from API)
    pub name: Option<String>,
    /// Avatar URL from GitHub
    pub avatar_url: String,
    /// GitHub profile URL
    pub profile_url: String,
    /// Number of contributions
    pub contributions: u32,
    /// Optional role/title (e.g., "Creator", "Maintainer")
    #[serde(default)]
    pub role: Option<String>,
}

impl Contributor {
    /// Get display name (falls back to login if name is not set)
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.login)
    }
}

/// Contributors data loaded from GitHub API
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContributorsData {
    /// When the data was last updated
    pub last_updated: String,
    /// Repository name
    pub repository: String,
    /// List of contributors
    pub contributors: Vec<Contributor>,
}

/// Repository owner and name
const REPO_OWNER: &str = "edgarhsanchez";
const REPO_NAME: &str = "dndgamerolls";

impl ContributorsData {
    /// Load contributors from GitHub API
    pub fn load() -> Self {
        println!("Fetching contributors from GitHub API...");

        match Self::fetch_from_github() {
            Ok(data) => {
                println!(
                    "Successfully loaded {} contributors from GitHub",
                    data.contributors.len()
                );
                data
            }
            Err(e) => {
                eprintln!("Failed to fetch contributors from GitHub: {}", e);
                Self::default_contributors()
            }
        }
    }

    /// Fetch contributors from GitHub API
    fn fetch_from_github() -> Result<Self, String> {
        let client = reqwest::blocking::Client::builder()
            .user_agent("DnDGameRolls/1.0")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        // Fetch contributors list
        let contributors_url = format!(
            "https://api.github.com/repos/{}/{}/contributors?per_page=100",
            REPO_OWNER, REPO_NAME
        );

        let response = client
            .get(&contributors_url)
            .send()
            .map_err(|e| format!("Failed to fetch contributors: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API returned status: {}", response.status()));
        }

        let github_contributors: Vec<GitHubContributor> = response
            .json()
            .map_err(|e| format!("Failed to parse contributors JSON: {}", e))?;

        // Convert to our format and fetch display names
        let mut contributors: Vec<Contributor> = Vec::new();

        for gc in github_contributors {
            // Try to fetch the user's display name
            let name = Self::fetch_user_name(&client, &gc.login);

            // Determine role (creator gets special role)
            let role = if gc.login.to_lowercase() == REPO_OWNER.to_lowercase() {
                Some("Creator".to_string())
            } else {
                None
            };

            contributors.push(Contributor {
                login: gc.login,
                name,
                avatar_url: format!("{}&s=64", gc.avatar_url), // Request 64px size
                profile_url: gc.html_url,
                contributions: gc.contributions,
                role,
            });
        }

        // Get current timestamp (simple format without chrono dependency)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| format!("fetched at unix:{}", d.as_secs()))
            .unwrap_or_else(|_| "now".to_string());

        Ok(Self {
            last_updated: now,
            repository: format!("{}/{}", REPO_OWNER, REPO_NAME),
            contributors,
        })
    }

    /// Fetch a user's display name from GitHub API
    fn fetch_user_name(client: &reqwest::blocking::Client, login: &str) -> Option<String> {
        let user_url = format!("https://api.github.com/users/{}", login);

        match client.get(&user_url).send() {
            Ok(response) if response.status().is_success() => {
                if let Ok(user) = response.json::<GitHubUser>() {
                    return user.name;
                }
            }
            _ => {}
        }

        None
    }

    /// Default contributors when API is unavailable
    fn default_contributors() -> Self {
        Self {
            last_updated: "offline".to_string(),
            repository: format!("{}/{}", REPO_OWNER, REPO_NAME),
            contributors: vec![Contributor {
                login: REPO_OWNER.to_string(),
                name: Some("Edgar Sanchez".to_string()),
                avatar_url: format!("https://github.com/{}.png?size=64", REPO_OWNER),
                profile_url: format!("https://github.com/{}", REPO_OWNER),
                contributions: 0,
                role: Some("Creator".to_string()),
            }],
        }
    }
}

/// Resource containing loaded contributors
#[derive(Resource, Default)]
pub struct ContributorsState {
    pub data: ContributorsData,
    pub loaded: bool,
}

// ============================================================================
// UI Components
// ============================================================================

/// Marker for the contributors screen root
#[derive(Component)]
pub struct ContributorsScreenRoot;

/// Marker for a contributor card/row
#[derive(Component)]
pub struct ContributorCard {
    pub index: usize,
}

/// Marker for the scrollable contributors list
#[derive(Component)]
pub struct ContributorsList;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contributor_display_name_with_name() {
        let contributor = Contributor {
            login: "testuser".to_string(),
            name: Some("Test User".to_string()),
            avatar_url: String::new(),
            profile_url: String::new(),
            contributions: 10,
            role: None,
        };
        assert_eq!(contributor.display_name(), "Test User");
    }

    #[test]
    fn test_contributor_display_name_without_name() {
        let contributor = Contributor {
            login: "testuser".to_string(),
            name: None,
            avatar_url: String::new(),
            profile_url: String::new(),
            contributions: 10,
            role: None,
        };
        assert_eq!(contributor.display_name(), "testuser");
    }

    #[test]
    fn test_contributors_data_default() {
        let data = ContributorsData::default();
        assert!(data.contributors.is_empty());
    }
}
