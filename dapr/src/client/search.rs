use std::collections::HashMap;
use std::time::Duration;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::{Client, DaprInterface};
use crate::dapr::proto::runtime::v1 as dapr_v1;
use crate::error::Error;

pub use crate::dapr::proto::runtime::v1::{
    IndexAck, IndexingMode, IndexingWaitTimeoutAction, SortOrder, TotalHitsRelation, search_query,
};
/// A `google.rpc.Status` as carried by [`FailedItem`] and batch vector query errors.
pub use crate::google::rpc::Status as RpcStatus;

/// A single index-ready document whose content is a JSON object
pub type SearchDocument = dapr_v1::SearchDocument;

/// A document matched by a search query
pub type SearchHit = dapr_v1::SearchHit;

/// A sort clause applied to search results
pub type SortClause = dapr_v1::SortClause;

/// Options controlling whether a write waits for the provider to complete it
pub type IndexingOptionsAlpha1 = dapr_v1::IndexingOptionsAlpha1;

/// An item that failed to be written
pub type FailedItem = dapr_v1::FailedItem;

/// A request to create a search index
pub type CreateIndexRequestAlpha1 = dapr_v1::CreateIndexRequestAlpha1;

/// A request to get a search index
pub type GetIndexRequestAlpha1 = dapr_v1::GetIndexRequestAlpha1;

/// A response containing a search index
pub type GetIndexResponseAlpha1 = dapr_v1::GetIndexResponseAlpha1;

/// A request to list the search indexes
pub type ListIndexesRequestAlpha1 = dapr_v1::ListIndexesRequestAlpha1;

/// A response containing the search indexes
pub type ListIndexesResponseAlpha1 = dapr_v1::ListIndexesResponseAlpha1;

/// A request to delete a search index
pub type DeleteIndexRequestAlpha1 = dapr_v1::DeleteIndexRequestAlpha1;

/// A request to index (upsert) documents
pub type IndexDocumentsRequestAlpha1 = dapr_v1::IndexDocumentsRequestAlpha1;

/// A response from indexing documents
pub type IndexDocumentsResponseAlpha1 = dapr_v1::IndexDocumentsResponseAlpha1;

/// A request to get documents by ID
pub type GetDocumentsRequestAlpha1 = dapr_v1::GetDocumentsRequestAlpha1;

/// A response containing the found documents
pub type GetDocumentsResponseAlpha1 = dapr_v1::GetDocumentsResponseAlpha1;

/// A request to delete documents by ID
pub type DeleteDocumentsRequestAlpha1 = dapr_v1::DeleteDocumentsRequestAlpha1;

/// A response from deleting documents
pub type DeleteDocumentsResponseAlpha1 = dapr_v1::DeleteDocumentsResponseAlpha1;

/// The query selecting and shaping the hits of a search
pub type SearchQuery = dapr_v1::SearchQuery;

/// A request to search an index
pub type SearchRequestAlpha1 = dapr_v1::SearchRequestAlpha1;

/// A response containing a page of search hits
pub type SearchResponseAlpha1 = dapr_v1::SearchResponseAlpha1;

impl SearchDocument {
    /// Creates a document by serializing `content`, which must serialize to a JSON object.
    pub fn from_json<I, T>(id: I, content: &T) -> Result<Self, Error>
    where
        I: Into<String>,
        T: Serialize + ?Sized,
    {
        let content = serde_json::to_vec(content).map_err(|_| Error::SerializationError)?;
        Ok(SearchDocument {
            id: id.into(),
            content,
            metadata: HashMap::new(),
        })
    }

    /// Sets the opaque caller metadata stored with the document.
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Deserializes the JSON content of the document.
    pub fn content_json<T: DeserializeOwned>(&self) -> Result<T, Error> {
        serde_json::from_slice(&self.content).map_err(|_| Error::SerializationError)
    }
}

impl IndexingOptionsAlpha1 {
    /// Waits for the provider to complete the write, for at most `wait_timeout`.
    ///
    /// # Arguments
    ///
    /// * `wait_timeout` - How long to wait; must be shorter than any RPC deadline
    /// * `on_wait_timeout` - What to do when the wait times out
    pub fn wait_for_completion(
        wait_timeout: Duration,
        on_wait_timeout: IndexingWaitTimeoutAction,
    ) -> Self {
        IndexingOptionsAlpha1 {
            mode: IndexingMode::WaitForCompletion.into(),
            wait_timeout: Some(prost_types::Duration {
                seconds: wait_timeout.as_secs().min(i64::MAX as u64) as i64,
                nanos: wait_timeout.subsec_nanos() as i32,
            }),
            on_wait_timeout: on_wait_timeout.into(),
        }
    }

    /// Returns once the provider has durably accepted the write.
    pub fn return_on_acceptance() -> Self {
        IndexingOptionsAlpha1 {
            mode: IndexingMode::ReturnOnAcceptance.into(),
            ..Default::default()
        }
    }
}

/// Converts a JSON object into a `google.protobuf.Struct`, as used by search and
/// vector filters, native queries and vector record metadata.
pub fn json_to_struct(value: Value) -> Result<prost_types::Struct, Error> {
    match json_to_value(value).kind {
        Some(prost_types::value::Kind::StructValue(s)) => Ok(s),
        _ => Err(Error::SerializationError),
    }
}

/// Converts a `google.protobuf.Struct` into a JSON object.
pub fn struct_to_json(value: &prost_types::Struct) -> Value {
    Value::Object(
        value
            .fields
            .iter()
            .map(|(k, v)| (k.clone(), value_to_json(v)))
            .collect(),
    )
}

fn json_to_value(value: Value) -> prost_types::Value {
    use prost_types::value::Kind;
    let kind = match value {
        Value::Null => Kind::NullValue(prost_types::NullValue::NullValue.into()),
        Value::Bool(b) => Kind::BoolValue(b),
        // Struct numbers are doubles; precision beyond 2^53 is lost, as with JSON.
        Value::Number(n) => Kind::NumberValue(n.as_f64().unwrap_or_default()),
        Value::String(s) => Kind::StringValue(s),
        Value::Array(a) => Kind::ListValue(prost_types::ListValue {
            values: a.into_iter().map(json_to_value).collect(),
        }),
        Value::Object(o) => Kind::StructValue(prost_types::Struct {
            fields: o.into_iter().map(|(k, v)| (k, json_to_value(v))).collect(),
        }),
    };
    prost_types::Value { kind: Some(kind) }
}

fn value_to_json(value: &prost_types::Value) -> Value {
    use prost_types::value::Kind;
    match &value.kind {
        None | Some(Kind::NullValue(_)) => Value::Null,
        Some(Kind::BoolValue(b)) => Value::Bool(*b),
        Some(Kind::NumberValue(n)) => serde_json::Number::from_f64(*n)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        Some(Kind::StringValue(s)) => Value::String(s.clone()),
        Some(Kind::ListValue(l)) => Value::Array(l.values.iter().map(value_to_json).collect()),
        Some(Kind::StructValue(s)) => struct_to_json(s),
    }
}

/// Builds a [`SearchQuery`] for [`Client::search_alpha1`].
pub struct SearchQueryBuilder {
    query: SearchQuery,
}

impl SearchQueryBuilder {
    /// Creates a search matching every document in the index, subject to any filter.
    pub fn all() -> Self {
        SearchQueryBuilder {
            query: SearchQuery::default(),
        }
    }

    /// Creates a full-text search.
    pub fn text(text: impl Into<String>) -> Self {
        Self::with_kind(search_query::Kind::Text(text.into()))
    }

    /// Creates a search using a component-native query.
    pub fn native(query: prost_types::Struct) -> Self {
        Self::with_kind(search_query::Kind::Native(query))
    }

    fn with_kind(kind: search_query::Kind) -> Self {
        SearchQueryBuilder {
            query: SearchQuery {
                kind: Some(kind),
                ..Default::default()
            },
        }
    }

    /// Sets the filter, expressed in the portable filter DSL.
    pub fn with_filter(mut self, filter: prost_types::Struct) -> Self {
        self.query.filter = Some(filter);
        self
    }

    /// Sets the maximum number of hits to return in the page.
    pub fn with_top_k(mut self, top_k: u32) -> Self {
        self.query.top_k = Some(top_k);
        self
    }

    /// Continues from the token returned by the previous page.
    pub fn with_continuation_token(mut self, token: impl Into<String>) -> Self {
        self.query.continuation_token = Some(token.into());
        self
    }

    /// Limits the content fields returned for each hit.
    pub fn with_return_fields(mut self, fields: Vec<String>) -> Self {
        self.query.return_fields = fields;
        self
    }

    /// Sets whether hit documents include their content.
    pub fn with_include_content(mut self, include: bool) -> Self {
        self.query.include_content = Some(include);
        self
    }

    /// Limits the content fields matched by a text query.
    pub fn with_search_fields(mut self, fields: Vec<String>) -> Self {
        self.query.search_fields = fields;
        self
    }

    /// Appends a sort clause.
    pub fn with_sort(mut self, field: impl Into<String>, order: SortOrder) -> Self {
        self.query.sort.push(SortClause {
            field: field.into(),
            order: order.into(),
        });
        self
    }

    /// Sets the fields to return highlighted fragments for.
    pub fn with_highlight_fields(mut self, fields: Vec<String>) -> Self {
        self.query.highlight_fields = fields;
        self
    }

    pub fn build(self) -> SearchQuery {
        self.query
    }
}

impl<T: DaprInterface> Client<T> {
    /// Creates a search index
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of the search store
    /// * `index` - The name of the index
    /// * `metadata` - Component-specific index settings
    pub async fn create_index_alpha1<S>(
        &mut self,
        store_name: S,
        index: S,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), Error>
    where
        S: Into<String>,
    {
        self.0
            .create_index_alpha1(CreateIndexRequestAlpha1 {
                store_name: store_name.into(),
                index: index.into(),
                metadata: metadata.unwrap_or_default(),
            })
            .await
    }

    /// Gets a search index
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of the search store
    /// * `index` - The name of the index
    /// * `metadata` - Any metadata pairs to include in the request
    pub async fn get_index_alpha1<S>(
        &mut self,
        store_name: S,
        index: S,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<GetIndexResponseAlpha1, Error>
    where
        S: Into<String>,
    {
        self.0
            .get_index_alpha1(GetIndexRequestAlpha1 {
                store_name: store_name.into(),
                index: index.into(),
                metadata: metadata.unwrap_or_default(),
            })
            .await
    }

    /// Lists the indexes in a search store
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of the search store
    /// * `metadata` - Any metadata pairs to include in the request
    pub async fn list_indexes_alpha1<S>(
        &mut self,
        store_name: S,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<ListIndexesResponseAlpha1, Error>
    where
        S: Into<String>,
    {
        self.0
            .list_indexes_alpha1(ListIndexesRequestAlpha1 {
                store_name: store_name.into(),
                metadata: metadata.unwrap_or_default(),
            })
            .await
    }

    /// Deletes a search index
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of the search store
    /// * `index` - The name of the index
    /// * `metadata` - Any metadata pairs to include in the request
    pub async fn delete_index_alpha1<S>(
        &mut self,
        store_name: S,
        index: S,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), Error>
    where
        S: Into<String>,
    {
        self.0
            .delete_index_alpha1(DeleteIndexRequestAlpha1 {
                store_name: store_name.into(),
                index: index.into(),
                metadata: metadata.unwrap_or_default(),
            })
            .await
    }

    /// Indexes (keyed upserts) documents
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of the search store
    /// * `index` - The name of the index
    /// * `documents` - The documents to index
    /// * `options` - Whether to wait for the provider to complete the write
    /// * `metadata` - Any metadata pairs to include in the request
    pub async fn index_documents_alpha1<S>(
        &mut self,
        store_name: S,
        index: S,
        documents: Vec<SearchDocument>,
        options: Option<IndexingOptionsAlpha1>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<IndexDocumentsResponseAlpha1, Error>
    where
        S: Into<String>,
    {
        self.0
            .index_documents_alpha1(IndexDocumentsRequestAlpha1 {
                store_name: store_name.into(),
                index: index.into(),
                documents,
                metadata: metadata.unwrap_or_default(),
                options,
            })
            .await
    }

    /// Gets documents by ID, in request order. IDs that do not exist are omitted.
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of the search store
    /// * `index` - The name of the index
    /// * `ids` - The IDs of the documents
    /// * `include_content` - Whether to return the document content
    /// * `metadata` - Any metadata pairs to include in the request
    pub async fn get_documents_alpha1<S, I>(
        &mut self,
        store_name: S,
        index: S,
        ids: I,
        include_content: bool,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<GetDocumentsResponseAlpha1, Error>
    where
        S: Into<String>,
        I: IntoIterator,
        I::Item: Into<String>,
    {
        self.0
            .get_documents_alpha1(GetDocumentsRequestAlpha1 {
                store_name: store_name.into(),
                index: index.into(),
                ids: ids.into_iter().map(Into::into).collect(),
                include_content: Some(include_content),
                metadata: metadata.unwrap_or_default(),
            })
            .await
    }

    /// Deletes documents by ID
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of the search store
    /// * `index` - The name of the index
    /// * `ids` - The IDs of the documents
    /// * `options` - Whether to wait for the provider to complete the write
    /// * `metadata` - Any metadata pairs to include in the request
    pub async fn delete_documents_alpha1<S, I>(
        &mut self,
        store_name: S,
        index: S,
        ids: I,
        options: Option<IndexingOptionsAlpha1>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<DeleteDocumentsResponseAlpha1, Error>
    where
        S: Into<String>,
        I: IntoIterator,
        I::Item: Into<String>,
    {
        self.0
            .delete_documents_alpha1(DeleteDocumentsRequestAlpha1 {
                store_name: store_name.into(),
                index: index.into(),
                ids: ids.into_iter().map(Into::into).collect(),
                metadata: metadata.unwrap_or_default(),
                options,
            })
            .await
    }

    /// Searches an index
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of the search store
    /// * `index` - The name of the index
    /// * `query` - The query, see [`SearchQueryBuilder`]; `None` matches every document
    /// * `metadata` - Any metadata pairs to include in the request
    pub async fn search_alpha1<S>(
        &mut self,
        store_name: S,
        index: S,
        query: Option<SearchQuery>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<SearchResponseAlpha1, Error>
    where
        S: Into<String>,
    {
        self.0
            .search_alpha1(SearchRequestAlpha1 {
                store_name: store_name.into(),
                index: index.into(),
                query,
                metadata: metadata.unwrap_or_default(),
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn json_struct_round_trip() {
        let value = json!({
            "$and": [
                {"category": {"$in": ["sedan", "suv"]}},
                {"price": {"$lte": 30000.5}},
                {"archived": {"$exists": false}},
                {"note": null}
            ]
        });
        let s = json_to_struct(value.clone()).unwrap();
        assert_eq!(struct_to_json(&s), value);
    }

    #[test]
    fn json_to_struct_rejects_non_object() {
        assert!(json_to_struct(json!([1, 2])).is_err());
        assert!(json_to_struct(json!("text")).is_err());
    }

    #[test]
    fn search_document_json() {
        #[derive(Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Car {
            model: String,
        }
        let car = Car {
            model: "roadster".to_string(),
        };
        let doc = SearchDocument::from_json("1", &car).unwrap();
        assert_eq!(doc.id, "1");
        assert_eq!(doc.content_json::<Car>().unwrap(), car);
    }

    #[test]
    fn indexing_options() {
        let opts = IndexingOptionsAlpha1::wait_for_completion(
            Duration::from_millis(1500),
            IndexingWaitTimeoutAction::FailRequest,
        );
        assert_eq!(opts.mode(), IndexingMode::WaitForCompletion);
        assert_eq!(
            opts.on_wait_timeout(),
            IndexingWaitTimeoutAction::FailRequest
        );
        let timeout = opts.wait_timeout.unwrap();
        assert_eq!((timeout.seconds, timeout.nanos), (1, 500_000_000));

        let opts = IndexingOptionsAlpha1::return_on_acceptance();
        assert_eq!(opts.mode(), IndexingMode::ReturnOnAcceptance);
        assert!(opts.wait_timeout.is_none());
    }

    #[test]
    fn search_query_builder() {
        let query = SearchQueryBuilder::text("electric")
            .with_top_k(5)
            .with_sort("price", SortOrder::Desc)
            .with_include_content(true)
            .build();
        assert_eq!(
            query.kind,
            Some(search_query::Kind::Text("electric".to_string()))
        );
        assert_eq!(query.top_k, Some(5));
        assert_eq!(query.sort.len(), 1);
        assert_eq!(query.sort[0].order(), SortOrder::Desc);
        assert_eq!(query.include_content, Some(true));
        assert!(query.continuation_token.is_none());

        let query = SearchQueryBuilder::all().build();
        assert!(query.kind.is_none());
        assert!(query.top_k.is_none());
    }
}
