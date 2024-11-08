use crate::constants;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum SearchType {
    Regex,
    Contains,
    Wildcard,
}
impl SearchType {
    pub fn sql_code(&self) -> i32 {
        match self {
            SearchType::Regex => constants::SEARCH_REGEX,
            SearchType::Contains => constants::SEARCH_CONTAINS,
            SearchType::Wildcard => constants::SEARCH_WILDCARD,
        }
    }
    pub fn from_sql_code(code: i32) -> Option<SearchType> {
        match code {
            constants::SEARCH_REGEX => Some(SearchType::Regex),
            constants::SEARCH_CONTAINS => Some(SearchType::Contains),
            constants::SEARCH_WILDCARD => Some(SearchType::Wildcard),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Search {
    pub id: i32,
    pub name: String,
    pub stype: SearchType,
    pub search: String,
    pub locations: Vec<String>,
}
impl Search {
    pub fn new(
        id: i32,
        name: String,
        stype: SearchType,
        search: String,
        locations: Vec<String>,
    ) -> Search {
        Search {
            id,
            name,
            stype,
            search,
            locations,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub client_id: String,
    pub client_name: String,
    pub search_id: i32,
    pub search_name: String,
    pub found: Vec<String>,
    pub location: String,
    pub started: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientSearchResult {
    pub search_id: i32,
    pub found: Vec<String>,
    pub location: String,
    pub started: DateTime<Utc>,
}
