use crate::model::SearchResult;
use crate::provider::ProviderRegistry;
use anyhow::Result;

pub fn search_all(
    registry: &ProviderRegistry,
    query: &str,
    limit: usize,
    provider_filter: Option<&[String]>,
) -> Result<Vec<SearchResult>> {
    registry.search_all(query, limit, provider_filter)
}
