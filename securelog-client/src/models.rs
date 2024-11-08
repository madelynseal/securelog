use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum SearchType {
    Regex,
    Contains,
    Wildcard,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Search {
    pub id: i32,
    pub name: String,
    pub stype: SearchType,
    pub search: String,
    pub locations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub search_id: i32,
    pub search_name: String,
    pub found: Vec<String>,
    pub location: String,
    pub started: DateTime<Utc>,
}
impl SearchResult {
    pub fn new(id: i32, name: &str, location: &str) -> SearchResult {
        SearchResult {
            search_id: id,
            search_name: name.to_owned(),
            location: location.to_string(),
            found: Vec::new(),
            started: Utc::now(),
        }
    }
}
