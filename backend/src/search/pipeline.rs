use anyhow::{Context, Result};
use elasticsearch::{BulkParts, Elasticsearch, IndexParts};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Semaphore;

#[derive(Debug, Error)]
pub enum IndexingError {
    #[error("Elasticsearch error: {0}")]
    ElasticsearchError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Batch processing error: {0}")]
    BatchError(String),
}

/// Indexing pipeline for bulk operations
pub struct IndexingPipeline {
    client: Arc<Elasticsearch>,
    index_name: String,
    batch_size: usize,
    max_concurrent_batches: usize,
}

impl IndexingPipeline {
    pub fn new(
        client: Arc<Elasticsearch>,
        index_name: String,
        batch_size: usize,
        max_concurrent_batches: usize,
    ) -> Self {
        Self {
            client,
            index_name,
            batch_size,
            max_concurrent_batches,
        }
    }

    /// Index a single document
    pub async fn index_document<T: Serialize>(&self, id: &str, document: &T) -> Result<(), IndexingError> {
        let response = self
            .client
            .index(IndexParts::IndexId(&self.index_name, id))
            .body(document)
            .send()
            .await
            .map_err(|e| IndexingError::ElasticsearchError(e.to_string()))?;

        if !response.status_code().is_success() {
            let error = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(IndexingError::ElasticsearchError(error));
        }

        Ok(())
    }

    /// Bulk index multiple documents
    pub async fn bulk_index<T: Serialize>(
        &self,
        documents: Vec<(String, T)>,
    ) -> Result<BulkIndexResult, IndexingError> {
        let mut result = BulkIndexResult::default();

        // Process in batches
        for chunk in documents.chunks(self.batch_size) {
            let batch_result = self.process_batch(chunk).await?;
            result.merge(batch_result);
        }

        Ok(result)
    }

    /// Process a single batch
    async fn process_batch<T: Serialize>(&self, batch: &[(String, T)]) -> Result<BulkIndexResult, IndexingError> {
        let mut body: Vec<String> = Vec::new();

        for (id, doc) in batch {
            body.push(serde_json::to_string(&json!({
                "index": {
                    "_index": self.index_name,
                    "_id": id
                }
            })).map_err(|e| IndexingError::SerializationError(e.to_string()))?);

            body.push(serde_json::to_string(&serde_json::to_value(doc)
                .map_err(|e| IndexingError::SerializationError(e.to_string()))?)
                .map_err(|e| IndexingError::SerializationError(e.to_string()))?);
        }

        let response = self
            .client
            .bulk(BulkParts::Index(&self.index_name))
            .body(body)
            .send()
            .await
            .map_err(|e| IndexingError::ElasticsearchError(e.to_string()))?;

        let response_body: BulkResponse = response
            .json()
            .await
            .map_err(|e| IndexingError::SerializationError(e.to_string()))?;

        let mut result = BulkIndexResult::default();

        for item in response_body.items {
            if let Some(index) = item.index {
                if index.status >= 200 && index.status < 300 {
                    result.success_count += 1;
                } else {
                    result.error_count += 1;
                    result.errors.push(format!(
                        "Document {}: {} - {}",
                        index._id,
                        index.status,
                        index.error.map(|e| e.reason).unwrap_or_default()
                    ));
                }
            }
        }

        Ok(result)
    }

    /// Delete a document by ID
    pub async fn delete_document(&self, id: &str) -> Result<(), IndexingError> {
        let response = self
            .client
            .delete(elasticsearch::DeleteParts::IndexId(&self.index_name, id))
            .send()
            .await
            .map_err(|e| IndexingError::ElasticsearchError(e.to_string()))?;

        if !response.status_code().is_success() && response.status_code().as_u16() != 404 {
            let error = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(IndexingError::ElasticsearchError(error));
        }

        Ok(())
    }
}

/// Result of bulk indexing operation
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BulkIndexResult {
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}

impl BulkIndexResult {
    fn merge(&mut self, other: BulkIndexResult) {
        self.success_count += other.success_count;
        self.error_count += other.error_count;
        self.errors.extend(other.errors);
    }
}

/// Bulk response structure
#[derive(Debug, Deserialize)]
struct BulkResponse {
    items: Vec<BulkItem>,
}

#[derive(Debug, Deserialize)]
struct BulkItem {
    index: Option<BulkIndexItem>,
}

#[derive(Debug, Deserialize)]
struct BulkIndexItem {
    _id: String,
    status: u16,
    error: Option<BulkError>,
}

#[derive(Debug, Deserialize)]
struct BulkError {
    reason: String,
}
