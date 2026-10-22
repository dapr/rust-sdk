use std::collections::HashMap;

use super::{Client, DaprInterface};
use crate::dapr::proto::runtime::v1 as dapr_v1;
use crate::error::Error;

pub use crate::dapr::proto::runtime::v1::{
    DistanceMetric, batch_query_result_alpha1, vector_query,
};

/// A dense vector with its payload and filterable metadata
pub type VectorRecord = dapr_v1::VectorRecord;

/// A record matched by a vector query
pub type VectorMatch = dapr_v1::VectorMatch;

/// A request to create a vector collection
pub type CreateCollectionRequestAlpha1 = dapr_v1::CreateCollectionRequestAlpha1;

/// A request to get a vector collection
pub type GetCollectionRequestAlpha1 = dapr_v1::GetCollectionRequestAlpha1;

/// A response containing a vector collection
pub type GetCollectionResponseAlpha1 = dapr_v1::GetCollectionResponseAlpha1;

/// A request to list the vector collections
pub type ListCollectionsRequestAlpha1 = dapr_v1::ListCollectionsRequestAlpha1;

/// A response containing the vector collections
pub type ListCollectionsResponseAlpha1 = dapr_v1::ListCollectionsResponseAlpha1;

/// A request to delete a vector collection
pub type DeleteCollectionRequestAlpha1 = dapr_v1::DeleteCollectionRequestAlpha1;

/// A request to upsert vector records
pub type UpsertVectorsRequestAlpha1 = dapr_v1::UpsertVectorsRequestAlpha1;

/// A response from upserting vector records
pub type UpsertVectorsResponseAlpha1 = dapr_v1::UpsertVectorsResponseAlpha1;

/// A request to delete vector records by ID
pub type DeleteVectorsRequestAlpha1 = dapr_v1::DeleteVectorsRequestAlpha1;

/// A response from deleting vector records
pub type DeleteVectorsResponseAlpha1 = dapr_v1::DeleteVectorsResponseAlpha1;

/// A request to get vector records by ID
pub type GetVectorsRequestAlpha1 = dapr_v1::GetVectorsRequestAlpha1;

/// A response containing the found vector records
pub type GetVectorsResponseAlpha1 = dapr_v1::GetVectorsResponseAlpha1;

/// The query selecting and shaping the matches of one vector query
pub type VectorQuery = dapr_v1::VectorQuery;

/// A request to query a collection for the nearest vector records
pub type QueryVectorsRequestAlpha1 = dapr_v1::QueryVectorsRequestAlpha1;

/// A response containing the matches of a vector query
pub type QueryVectorsResponseAlpha1 = dapr_v1::QueryVectorsResponseAlpha1;

/// A request to run multiple vector queries against a collection
pub type BatchQueryVectorsRequestAlpha1 = dapr_v1::BatchQueryVectorsRequestAlpha1;

/// The outcome of one query in a batch
pub type BatchQueryResultAlpha1 = dapr_v1::BatchQueryResultAlpha1;

/// A response containing one result per batched query, in request order
pub type BatchQueryVectorsResponseAlpha1 = dapr_v1::BatchQueryVectorsResponseAlpha1;

impl VectorRecord {
    /// Creates a record from a dense vector.
    pub fn new(id: impl Into<String>, values: Vec<f32>) -> Self {
        VectorRecord {
            id: id.into(),
            values,
            ..Default::default()
        }
    }

    /// Sets the opaque payload stored with the record.
    pub fn with_payload(mut self, payload: Vec<u8>) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Sets the filterable metadata, see [`super::json_to_struct`].
    pub fn with_metadata(mut self, metadata: prost_types::Struct) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Builds a [`VectorQuery`] for [`Client::query_vectors_alpha1`] and
/// [`Client::batch_query_vectors_alpha1`].
pub struct VectorQueryBuilder {
    query: VectorQuery,
}

impl VectorQueryBuilder {
    /// Queries for the records nearest to `values`, which must not be empty.
    pub fn vector(values: Vec<f32>) -> Self {
        Self::with_kind(vector_query::Kind::Vector(VectorRecord {
            values,
            ..Default::default()
        }))
    }

    /// Queries for the records nearest to the stored record `id`.
    pub fn by_id(id: impl Into<String>) -> Self {
        Self::with_kind(vector_query::Kind::ById(id.into()))
    }

    fn with_kind(kind: vector_query::Kind) -> Self {
        VectorQueryBuilder {
            query: VectorQuery {
                kind: Some(kind),
                ..Default::default()
            },
        }
    }

    /// Sets the maximum number of matches to return.
    pub fn with_top_k(mut self, top_k: u32) -> Self {
        self.query.top_k = Some(top_k);
        self
    }

    /// Sets the metadata filter, expressed in the portable filter DSL.
    pub fn with_filter(mut self, filter: prost_types::Struct) -> Self {
        self.query.filter = Some(filter);
        self
    }

    /// Sets whether matches include their vector values.
    pub fn with_include_values(mut self, include: bool) -> Self {
        self.query.include_values = Some(include);
        self
    }

    /// Sets whether matches include their payload.
    pub fn with_include_payload(mut self, include: bool) -> Self {
        self.query.include_payload = Some(include);
        self
    }

    /// Sets the metric used to score matches, overriding the collection's metric.
    pub fn with_metric(mut self, metric: DistanceMetric) -> Self {
        self.query.metric = metric.into();
        self
    }

    /// Sets the inclusive score cutoff. Matches must score at least this value for
    /// cosine and dot product, or at most this value for euclidean distance.
    pub fn with_score_threshold(mut self, threshold: f64) -> Self {
        self.query.score_threshold = Some(threshold);
        self
    }

    pub fn build(self) -> VectorQuery {
        self.query
    }
}

impl<T: DaprInterface> Client<T> {
    /// Creates a vector collection
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of the vector store
    /// * `collection` - The name of the collection
    /// * `dimensions` - The length of every vector stored in the collection
    /// * `metric` - The distance metric, `Unspecified` selects the component default
    /// * `metadata` - Component-specific collection settings
    pub async fn create_collection_alpha1<S>(
        &mut self,
        store_name: S,
        collection: S,
        dimensions: u32,
        metric: DistanceMetric,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), Error>
    where
        S: Into<String>,
    {
        self.0
            .create_collection_alpha1(CreateCollectionRequestAlpha1 {
                store_name: store_name.into(),
                collection: collection.into(),
                metadata: metadata.unwrap_or_default(),
                dimensions,
                metric: metric.into(),
            })
            .await
    }

    /// Gets a vector collection
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of the vector store
    /// * `collection` - The name of the collection
    /// * `metadata` - Any metadata pairs to include in the request
    pub async fn get_collection_alpha1<S>(
        &mut self,
        store_name: S,
        collection: S,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<GetCollectionResponseAlpha1, Error>
    where
        S: Into<String>,
    {
        self.0
            .get_collection_alpha1(GetCollectionRequestAlpha1 {
                store_name: store_name.into(),
                collection: collection.into(),
                metadata: metadata.unwrap_or_default(),
            })
            .await
    }

    /// Lists the collections in a vector store
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of the vector store
    /// * `metadata` - Any metadata pairs to include in the request
    pub async fn list_collections_alpha1<S>(
        &mut self,
        store_name: S,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<ListCollectionsResponseAlpha1, Error>
    where
        S: Into<String>,
    {
        self.0
            .list_collections_alpha1(ListCollectionsRequestAlpha1 {
                store_name: store_name.into(),
                metadata: metadata.unwrap_or_default(),
            })
            .await
    }

    /// Deletes a vector collection
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of the vector store
    /// * `collection` - The name of the collection
    /// * `metadata` - Any metadata pairs to include in the request
    pub async fn delete_collection_alpha1<S>(
        &mut self,
        store_name: S,
        collection: S,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), Error>
    where
        S: Into<String>,
    {
        self.0
            .delete_collection_alpha1(DeleteCollectionRequestAlpha1 {
                store_name: store_name.into(),
                collection: collection.into(),
                metadata: metadata.unwrap_or_default(),
            })
            .await
    }

    /// Upserts (keyed) vector records
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of the vector store
    /// * `collection` - The name of the collection
    /// * `records` - The records to upsert
    /// * `options` - Whether to wait for the provider to complete the write
    /// * `metadata` - Any metadata pairs to include in the request
    pub async fn upsert_vectors_alpha1<S>(
        &mut self,
        store_name: S,
        collection: S,
        records: Vec<VectorRecord>,
        options: Option<super::IndexingOptionsAlpha1>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<UpsertVectorsResponseAlpha1, Error>
    where
        S: Into<String>,
    {
        self.0
            .upsert_vectors_alpha1(UpsertVectorsRequestAlpha1 {
                store_name: store_name.into(),
                collection: collection.into(),
                records,
                metadata: metadata.unwrap_or_default(),
                options,
            })
            .await
    }

    /// Deletes vector records by ID
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of the vector store
    /// * `collection` - The name of the collection
    /// * `ids` - The IDs of the records
    /// * `options` - Whether to wait for the provider to complete the write
    /// * `metadata` - Any metadata pairs to include in the request
    pub async fn delete_vectors_alpha1<S, I>(
        &mut self,
        store_name: S,
        collection: S,
        ids: I,
        options: Option<super::IndexingOptionsAlpha1>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<DeleteVectorsResponseAlpha1, Error>
    where
        S: Into<String>,
        I: IntoIterator,
        I::Item: Into<String>,
    {
        self.0
            .delete_vectors_alpha1(DeleteVectorsRequestAlpha1 {
                store_name: store_name.into(),
                collection: collection.into(),
                ids: ids.into_iter().map(Into::into).collect(),
                metadata: metadata.unwrap_or_default(),
                options,
            })
            .await
    }

    /// Gets vector records by ID, in request order. IDs that do not exist are omitted.
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of the vector store
    /// * `collection` - The name of the collection
    /// * `ids` - The IDs of the records
    /// * `include_values` - Whether to return the vector values
    /// * `metadata` - Any metadata pairs to include in the request
    pub async fn get_vectors_alpha1<S, I>(
        &mut self,
        store_name: S,
        collection: S,
        ids: I,
        include_values: bool,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<GetVectorsResponseAlpha1, Error>
    where
        S: Into<String>,
        I: IntoIterator,
        I::Item: Into<String>,
    {
        self.0
            .get_vectors_alpha1(GetVectorsRequestAlpha1 {
                store_name: store_name.into(),
                collection: collection.into(),
                ids: ids.into_iter().map(Into::into).collect(),
                include_values: Some(include_values),
                metadata: metadata.unwrap_or_default(),
            })
            .await
    }

    /// Queries a collection for the nearest vector records
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of the vector store
    /// * `collection` - The name of the collection
    /// * `query` - The query, see [`VectorQueryBuilder`]
    /// * `metadata` - Any metadata pairs to include in the request
    pub async fn query_vectors_alpha1<S>(
        &mut self,
        store_name: S,
        collection: S,
        query: VectorQuery,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<QueryVectorsResponseAlpha1, Error>
    where
        S: Into<String>,
    {
        self.0
            .query_vectors_alpha1(QueryVectorsRequestAlpha1 {
                store_name: store_name.into(),
                collection: collection.into(),
                query: Some(query),
                metadata: metadata.unwrap_or_default(),
            })
            .await
    }

    /// Runs multiple queries against a collection, each succeeding or failing on its own
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of the vector store
    /// * `collection` - The name of the collection
    /// * `queries` - The queries, see [`VectorQueryBuilder`]
    /// * `metadata` - Any metadata pairs to include in the request, applied to every query
    pub async fn batch_query_vectors_alpha1<S>(
        &mut self,
        store_name: S,
        collection: S,
        queries: Vec<VectorQuery>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<BatchQueryVectorsResponseAlpha1, Error>
    where
        S: Into<String>,
    {
        self.0
            .batch_query_vectors_alpha1(BatchQueryVectorsRequestAlpha1 {
                store_name: store_name.into(),
                collection: collection.into(),
                queries,
                metadata: metadata.unwrap_or_default(),
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_query_builder() {
        let query = VectorQueryBuilder::vector(vec![0.1, 0.2])
            .with_top_k(3)
            .with_metric(DistanceMetric::Cosine)
            .with_score_threshold(0.5)
            .with_include_payload(true)
            .build();
        match &query.kind {
            Some(vector_query::Kind::Vector(v)) => {
                assert_eq!(v.values, vec![0.1, 0.2])
            }
            other => panic!("unexpected query {other:?}"),
        }
        assert_eq!(query.top_k, Some(3));
        assert_eq!(query.metric(), DistanceMetric::Cosine);
        assert_eq!(query.score_threshold, Some(0.5));
        assert_eq!(query.include_payload, Some(true));
        assert!(query.include_values.is_none());

        let query = VectorQueryBuilder::by_id("car-1").build();
        assert_eq!(
            query.kind,
            Some(vector_query::Kind::ById("car-1".to_string()))
        );
    }

    #[test]
    fn vector_record() {
        let record = VectorRecord::new("car-1", vec![1.0, 0.0]).with_payload(b"p".to_vec());
        assert_eq!(record.id, "car-1");
        assert_eq!(record.payload.as_deref(), Some(&b"p"[..]));
        assert!(record.metadata.is_none());
    }
}
