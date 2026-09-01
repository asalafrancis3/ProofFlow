use proofflow_backend::search::{
    field_builders, Facet, FacetType, FacetedSearch, IndexConfig, IndexMapping, IndexingPipeline, SearchClient,
    SearchClientConfig, SearchFilter, SearchIndex, SearchQueryBuilder,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestDocument {
    id: String,
    title: String,
    description: String,
    status: String,
    created_at: String,
}

#[tokio::test]
async fn test_search_client_creation() {
    let config = SearchClientConfig::default();
    let result = SearchClient::new(config);
    assert!(result.is_ok());
}

#[test]
fn test_index_mapping_builder() {
    let mapping = IndexMapping::new()
        .add_field("title".to_string(), field_builders::text_with_keyword())
        .add_field("status".to_string(), field_builders::keyword_field())
        .add_field("created_at".to_string(), field_builders::date_field())
        .add_field("count".to_string(), field_builders::integer_field())
        .add_field("active".to_string(), field_builders::boolean_field());

    assert_eq!(mapping.properties.len(), 5);
    assert!(mapping.properties.contains_key("title"));
    assert!(mapping.properties.contains_key("status"));
}

#[test]
fn test_search_query_builder() {
    let query = SearchQueryBuilder::new()
        .match_query("title", "test document")
        .from(0)
        .size(20)
        .sort("created_at", "desc")
        .build();

    assert_eq!(query.from, 0);
    assert_eq!(query.size, 20);
    assert!(query.sort.is_some());
}

#[test]
fn test_multi_match_query() {
    let query = SearchQueryBuilder::new()
        .multi_match(vec!["title".to_string(), "description".to_string()], "search term")
        .build();

    // Verify the query was built correctly
    assert_eq!(query.from, 0);
    assert_eq!(query.size, 20);
}

#[test]
fn test_bool_query_builder() {
    use proofflow_backend::search::query_builder::QueryType;

    let must_queries = vec![QueryType::Term {
        field: "status".to_string(),
        value: json!("active"),
    }];

    let should_queries = vec![QueryType::Match {
        field: "title".to_string(),
        query: "important".to_string(),
    }];

    let query = SearchQueryBuilder::new()
        .bool_query(must_queries, should_queries, vec![])
        .build();

    // Verify the boolean query structure
    match query.query {
        QueryType::Bool { must, should, must_not } => {
            assert_eq!(must.len(), 1);
            assert_eq!(should.len(), 1);
            assert_eq!(must_not.len(), 0);
        }
        _ => panic!("Expected Bool query"),
    }
}

#[test]
fn test_search_filters() {
    let filter = SearchFilter::term("status", json!("active"));
    assert!(matches!(
        filter.filter,
        proofflow_backend::search::FilterType::Term { .. }
    ));

    let range_filter = SearchFilter::range("price", Some(json!(10)), Some(json!(100)));
    assert!(matches!(
        range_filter.filter,
        proofflow_backend::search::FilterType::Range { .. }
    ));

    let exists_filter = SearchFilter::exists("email");
    assert!(matches!(
        exists_filter.filter,
        proofflow_backend::search::FilterType::Exists { .. }
    ));
}

#[test]
fn test_faceted_search() {
    let facets = vec![
        Facet {
            field: "status".to_string(),
            size: 10,
            facet_type: FacetType::Terms,
        },
        Facet {
            field: "created_at".to_string(),
            size: 10,
            facet_type: FacetType::DateHistogram {
                interval: "month".to_string(),
            },
        },
    ];

    let aggregations = FacetedSearch::build_aggregations(&facets);
    assert_eq!(aggregations.len(), 2);
    assert!(aggregations.contains_key("status"));
    assert!(aggregations.contains_key("created_at"));
}

#[test]
fn test_query_to_elasticsearch_json() {
    let builder = SearchQueryBuilder::new()
        .match_query("title", "test")
        .from(10)
        .size(50)
        .sort("_score", "desc")
        .highlight(vec!["title".to_string()]);

    let json = builder.to_elasticsearch_json();

    assert_eq!(json["from"], 10);
    assert_eq!(json["size"], 50);
    assert!(json["query"].is_object());
    assert!(json["sort"].is_array());
    assert!(json["highlight"].is_object());
}

#[test]
fn test_pagination() {
    let query = SearchQueryBuilder::new().from(100).size(25).build();

    assert_eq!(query.from, 100);
    assert_eq!(query.size, 25);
}

#[test]
fn test_aggregation_builder() {
    let query = SearchQueryBuilder::new()
        .aggregation(
            "status_count",
            json!({
                "terms": {
                    "field": "status.keyword",
                    "size": 10
                }
            }),
        )
        .build();

    assert!(query.aggregations.is_some());
    let aggs = query.aggregations.unwrap();
    assert!(aggs.contains_key("status_count"));
}

#[test]
fn test_source_filtering() {
    let query = SearchQueryBuilder::new()
        .source(vec!["title".to_string(), "description".to_string()])
        .build();

    assert!(query.source.is_some());
    let source = query.source.unwrap();
    assert_eq!(source.len(), 2);
    assert!(source.contains(&"title".to_string()));
}
