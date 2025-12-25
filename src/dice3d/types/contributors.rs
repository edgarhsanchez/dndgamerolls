//! Contributors data types
//!
//! This module contains types for loading and displaying contributors.
//!
//! To satisfy the project constraint of "no JSON", contributor data is loaded from
//! a bundled RON asset (see `assets/contributors.ron`).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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
    /// Load contributors from the bundled asset.
    pub fn load() -> Self {
        let text = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/contributors.ron"
        ));
        match ron::from_str::<ContributorsData>(text) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to parse contributors asset: {}", e);
                Self::default_contributors()
            }
        }
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
