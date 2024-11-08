use crate::models::{Search, SearchResult, SearchType};
use crate::webclient::{self};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("SearchError(File read only: {0})")]
    FileReadOnly(String),

    #[error("SearchError(IO({0}))")]
    IO(#[from] std::io::Error),

    #[error("SearchError(Web({0}))")]
    Web(#[from] crate::webclient::WebError),

    #[error("SearchError(Regex({0}))")]
    Regex(#[from] regex::Error),
}

type Result<T> = std::result::Result<T, SearchError>;

pub fn run_once() -> Result<()> {
    let searches = webclient::get_searches()?;

    for search in &searches {
        let mut results: Vec<SearchResult> = Vec::new();

        for location in &search.locations {
            match run_search(location, search) {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    warn!("error running search {}: {}", search.id, e);
                }
            }
        }
        if !results.is_empty() {
            webclient::send_search_results(&results)?;
        }
    }

    Ok(())
}

pub fn run_search(path: &str, search: &Search) -> Result<SearchResult> {
    check_file_can_read(path)?;

    match search.stype {
        SearchType::Contains => run_search_contains(path, search),
        SearchType::Regex => run_search_regex(path, search),
        SearchType::Wildcard => run_search_wildcard(path, search),
    }
}

fn run_search_contains(path: &str, search: &Search) -> Result<SearchResult> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut results = SearchResult::new(search.id, &search.name, path);
    for (_index, line) in reader.lines().enumerate() {
        let line = line?;
        if line.contains(&search.search) {
            results.found.push(line);
        }
    }

    Ok(results)
}

fn run_search_regex(path: &str, search: &Search) -> Result<SearchResult> {
    use regex::Regex;
    let rgx = Regex::new(&search.search)?;
    let mut results = SearchResult::new(search.id, &search.name, path);

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for (_index, line) in reader.lines().enumerate() {
        let line = line?;
        if rgx.is_match(&line) {
            results.found.push(line);
        }
    }

    Ok(results)
}

fn run_search_wildcard(path: &str, search: &Search) -> Result<SearchResult> {
    use wildmatch::WildMatch;
    let wmatch = WildMatch::new(&search.search);

    let mut results = SearchResult::new(search.id, &search.name, path);

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for (_index, line) in reader.lines().enumerate() {
        let line = line?;
        if wmatch.matches(&line) {
            results.found.push(line);
        }
    }

    Ok(results)
}

fn check_file_can_read(path: &str) -> Result<()> {
    if let Ok(metadata) = fs::metadata(path) {
        if metadata.permissions().readonly() {
            Err(SearchError::FileReadOnly(path.to_string()))
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}
