use dapr::client::{
    IndexingOptionsAlpha1, IndexingWaitTimeoutAction, SearchDocument, SearchRequestBuilder,
    SortOrder, json_to_struct,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;

const STORE_NAME: &str = "search";
const INDEX: &str = "cars";

#[derive(Serialize, Deserialize, Debug)]
struct Car {
    model: String,
    category: String,
    price: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sleep to allow for the server to become available
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Connect using env vars
    let mut client = dapr::Client::new().await?;

    // Index settings are component specific. Meilisearch only filters and sorts
    // on declared attributes.
    let settings = HashMap::from([
        (
            "filterableAttributes".to_string(),
            "category,price".to_string(),
        ),
        ("sortableAttributes".to_string(), "price".to_string()),
    ]);
    client
        .create_index_alpha1(STORE_NAME, INDEX, Some(settings))
        .await?;
    println!("index created: {INDEX}");

    let cars = [
        ("1", "electric roadster", "sports", 90000),
        ("2", "electric hatchback", "compact", 30000),
        ("3", "diesel pickup", "truck", 45000),
    ];
    let documents = cars
        .into_iter()
        .map(|(id, model, category, price)| {
            SearchDocument::from_json(
                id,
                &Car {
                    model: model.to_string(),
                    category: category.to_string(),
                    price,
                },
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Wait for the documents to be indexed so they are immediately searchable
    let options = IndexingOptionsAlpha1::wait_for_completion(
        Duration::from_secs(10),
        IndexingWaitTimeoutAction::FailRequest,
    );
    let response = client
        .index_documents_alpha1(STORE_NAME, INDEX, documents, Some(options), None)
        .await?;
    println!(
        "indexed documents: ack={:?} failed={}",
        response.ack(),
        response.failed_items.len()
    );

    // Full-text search, filtered and sorted
    let request = SearchRequestBuilder::text("electric")
        .with_filter(json_to_struct(json!({"price": {"$lt": 100000}}))?)
        .with_sort("price", SortOrder::Asc)
        .with_include_content(true)
        .with_top_k(10)
        .build();
    let response = client.search_alpha1(STORE_NAME, INDEX, request).await?;
    for hit in response.hits {
        if let Some(document) = hit.document {
            let car: Car = document.content_json()?;
            println!("search hit: {} {}", document.id, car.model);
        }
    }

    let response = client
        .get_documents_alpha1(STORE_NAME, INDEX, ["3"], true, None)
        .await?;
    for document in response.documents {
        let car: Car = document.content_json()?;
        println!("got document: {} {}", document.id, car.model);
    }

    client
        .delete_documents_alpha1(STORE_NAME, INDEX, ["1", "2", "3"], None, None)
        .await?;
    client.delete_index_alpha1(STORE_NAME, INDEX, None).await?;
    println!("index deleted: {INDEX}");

    Ok(())
}
