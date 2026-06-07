//! B9: Vector DB Integration Tests
//!
//! Tests the vector storage layer: embedding, search, sync, delete.

use std::sync::Arc;

use wb_storage::vector::{
    EmbeddingProvider, InMemoryVectorStore, MockEmbedding, SemanticSearch,
    VectorStore, VectorSync,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn create_store() -> InMemoryVectorStore {
    let embedding = Arc::new(MockEmbedding::new(128));
    InMemoryVectorStore::new(embedding)
}

async fn create_populated_store() -> InMemoryVectorStore {
    let store = create_store();
    store
        .upsert("rust", "Rust is a systems programming language")
        .await
        .unwrap();
    store
        .upsert("python", "Python is great for data science and ML")
        .await
        .unwrap();
    store
        .upsert("js", "JavaScript is used for web development")
        .await
        .unwrap();
    store
        .upsert("rust-perf", "Rust focuses on safety and performance")
        .await
        .unwrap();
    store
}

// ---------------------------------------------------------------------------
// B9-01: Embed
// ---------------------------------------------------------------------------

#[tokio::test]
async fn b9_01_embed_dimensions() {
    let engine = MockEmbedding::new(64);
    let vector = engine.embed("test text").await.unwrap();
    assert_eq!(vector.len(), 64);
}

#[tokio::test]
async fn b9_01_embed_deterministic() {
    let engine = MockEmbedding::default_128();
    let vec1 = engine.embed("hello world").await.unwrap();
    let vec2 = engine.embed("hello world").await.unwrap();
    assert_eq!(vec1, vec2, "Same text should produce same embedding");
}

#[tokio::test]
async fn b9_01_embed_different_texts() {
    let engine = MockEmbedding::default_128();
    let vec1 = engine.embed("hello").await.unwrap();
    let vec2 = engine.embed("world").await.unwrap();
    assert_ne!(vec1, vec2, "Different text should produce different embeddings");
}

#[tokio::test]
async fn b9_01_embed_normalized() {
    let engine = MockEmbedding::new(128);
    let vector = engine.embed("normalize test").await.unwrap();
    let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-6, "Vector should be L2 normalized");
}

#[tokio::test]
async fn b9_01_upsert_and_get() {
    let store = create_store();
    store.upsert("doc1", "hello world").await.unwrap();

    let embedding = store.get("doc1").await.unwrap();
    assert!(embedding.is_some());
    assert_eq!(embedding.unwrap().len(), 128);
}

#[tokio::test]
async fn b9_01_upsert_updates_existing() {
    let store = create_store();
    store.upsert("doc1", "original content").await.unwrap();
    store.upsert("doc1", "updated content").await.unwrap();

    assert_eq!(store.count().await, 1, "Should still have 1 doc after update");
}

// ---------------------------------------------------------------------------
// B9-02: Search
// ---------------------------------------------------------------------------

#[tokio::test]
async fn b9_02_search_top_k() {
    let store = create_populated_store().await;

    let results = store.search("programming", 2).await.unwrap();
    assert_eq!(results.len(), 2);
    // Results should be sorted by score descending
    assert!(results[0].score >= results[1].score);
}

#[tokio::test]
async fn b9_02_search_returns_content_snippet() {
    let store = create_populated_store().await;

    let results = store.search("rust", 1).await.unwrap();
    assert_eq!(results.len(), 1);
    assert!(
        !results[0].content_snippet.is_empty(),
        "Should include content snippet"
    );
}

#[tokio::test]
async fn b9_02_search_cosine_similarity() {
    let store = create_store();

    // Identical vectors should have similarity ~1.0
    store.upsert("a", "test").await.unwrap();
    let results = store.search("test", 1).await.unwrap();
    assert_eq!(results.len(), 1);
    assert!(
        (results[0].score - 1.0).abs() < 1e-6,
        "Identical text should have similarity ~1.0"
    );
}

#[tokio::test]
async fn b9_02_search_empty_store() {
    let store = create_store();
    let results = store.search("anything", 5).await.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn b9_02_search_with_threshold() {
    let store = create_populated_store().await;
    let search = SemanticSearch::new(store);

    let results = search.search_with_threshold("rust", 10, 0.5).await.unwrap();
    for result in &results {
        assert!(result.score >= 0.5, "All results should be above threshold");
    }
}

#[tokio::test]
async fn b9_02_multi_query_search() {
    let store = create_populated_store().await;
    let search = SemanticSearch::new(store);

    let results = search
        .multi_query_search(&["rust", "performance"], 3)
        .await
        .unwrap();

    // Should return deduplicated results
    let doc_ids: Vec<&str> = results.iter().map(|r| r.doc_id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = doc_ids.iter().copied().collect();
    assert_eq!(doc_ids.len(), unique.len(), "Results should be deduplicated");
}

#[tokio::test]
async fn b9_02_similar_docs() {
    let store = create_populated_store().await;

    let results = store.similar("rust", 2).await.unwrap();
    assert_eq!(results.len(), 2);
    // "rust" itself should not be in results
    assert!(!results.iter().any(|r| r.doc_id == "rust"));
}

#[tokio::test]
async fn b9_02_similar_nonexistent() {
    let store = create_populated_store().await;
    let results = store.similar("nonexistent", 5).await.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn b9_02_rag_context() {
    let store = create_populated_store().await;

    let context = store.rag_context("programming", 500).await.unwrap();
    assert!(!context.is_empty(), "RAG context should not be empty");
}

#[tokio::test]
async fn b9_02_rag_context_respects_token_limit() {
    let store = create_populated_store().await;

    let context = store.rag_context("programming", 5).await.unwrap();
    // With very small token limit, context should be truncated
    // (may be empty or very short)
    assert!(context.len() < 1000, "Context should be truncated");
}

// ---------------------------------------------------------------------------
// B9-03: Sync
// ---------------------------------------------------------------------------

#[tokio::test]
async fn b9_03_sync_changed_existing() {
    let embedding = Arc::new(MockEmbedding::new(128));
    let store = InMemoryVectorStore::new(embedding);
    let sync = VectorSync::new(store.clone());

    store.upsert("doc1", "hello world").await.unwrap();

    let report = sync
        .sync_changed(&["doc1".to_string()])
        .await
        .unwrap();
    assert_eq!(report.synced_count, 1);
    assert_eq!(report.failed_count, 0);
    assert_eq!(report.skipped_count, 0);
}

#[tokio::test]
async fn b9_03_sync_changed_nonexistent() {
    let embedding = Arc::new(MockEmbedding::new(128));
    let store = InMemoryVectorStore::new(embedding);
    let sync = VectorSync::new(store);

    let report = sync
        .sync_changed(&["nonexistent".to_string()])
        .await
        .unwrap();
    assert_eq!(report.synced_count, 0);
    assert_eq!(report.skipped_count, 1);
}

#[tokio::test]
async fn b9_03_sync_changed_mixed() {
    let embedding = Arc::new(MockEmbedding::new(128));
    let store = InMemoryVectorStore::new(embedding);
    let sync = VectorSync::new(store.clone());

    store.upsert("doc1", "hello").await.unwrap();

    let report = sync
        .sync_changed(&["doc1".to_string(), "nonexistent".to_string()])
        .await
        .unwrap();
    assert_eq!(report.synced_count, 1);
    assert_eq!(report.skipped_count, 1);
}

#[tokio::test]
async fn b9_03_batch_reembed() {
    let embedding = Arc::new(MockEmbedding::new(128));
    let store = InMemoryVectorStore::new(embedding);
    let sync = VectorSync::new(store.clone());

    let docs = vec![("doc1", "content 1"), ("doc2", "content 2"), ("doc3", "content 3")];
    let report = sync.batch_reembed(&docs).await.unwrap();

    assert_eq!(report.synced_count, 3);
    assert_eq!(report.failed_count, 0);
    assert_eq!(store.count().await, 3);
}

#[tokio::test]
async fn b9_03_batch_reembed_empty() {
    let embedding = Arc::new(MockEmbedding::new(128));
    let store = InMemoryVectorStore::new(embedding);
    let sync = VectorSync::new(store);

    let docs = vec![];
    let report = sync.batch_reembed(&docs).await.unwrap();
    assert_eq!(report.synced_count, 0);
}

#[tokio::test]
async fn b9_03_reembed_updates_content() {
    let embedding = Arc::new(MockEmbedding::new(128));
    let store = InMemoryVectorStore::new(embedding);
    let sync = VectorSync::new(store.clone());

    sync.reembed("doc1", "original content").await.unwrap();
    assert_eq!(sync.count().await, 1);

    sync.reembed("doc1", "updated content").await.unwrap();
    assert_eq!(sync.count().await, 1, "Should still be 1 after update");
}

// ---------------------------------------------------------------------------
// B9-04: Delete
// ---------------------------------------------------------------------------

#[tokio::test]
async fn b9_04_remove_existing() {
    let store = create_store();
    store.upsert("doc1", "hello").await.unwrap();

    let removed = store.remove("doc1").await.unwrap();
    assert!(removed, "Should return true when removing existing doc");

    let embedding = store.get("doc1").await.unwrap();
    assert!(embedding.is_none(), "Doc should be gone after removal");
}

#[tokio::test]
async fn b9_04_remove_nonexistent() {
    let store = create_store();
    let removed = store.remove("nonexistent").await.unwrap();
    assert!(!removed, "Should return false when removing non-existent doc");
}

#[tokio::test]
async fn b9_04_remove_updates_count() {
    let store = create_store();
    store.upsert("doc1", "a").await.unwrap();
    store.upsert("doc2", "b").await.unwrap();
    assert_eq!(store.count().await, 2);

    store.remove("doc1").await.unwrap();
    assert_eq!(store.count().await, 1);
}

#[tokio::test]
async fn b9_04_remove_does_not_affect_others() {
    let store = create_store();
    store.upsert("doc1", "hello").await.unwrap();
    store.upsert("doc2", "world").await.unwrap();

    store.remove("doc1").await.unwrap();

    let doc2 = store.get("doc2").await.unwrap();
    assert!(doc2.is_some(), "Other docs should not be affected");
}

#[tokio::test]
async fn b9_04_remove_then_reinsert() {
    let store = create_store();
    store.upsert("doc1", "original").await.unwrap();
    store.remove("doc1").await.unwrap();

    // Re-insert with same id
    store.upsert("doc1", "new content").await.unwrap();
    assert_eq!(store.count().await, 1);

    let embedding = store.get("doc1").await.unwrap();
    assert!(embedding.is_some());
}

#[tokio::test]
async fn b9_04_count_operations() {
    let store = create_store();
    assert_eq!(store.count().await, 0);

    store.upsert("doc1", "a").await.unwrap();
    store.upsert("doc2", "b").await.unwrap();
    assert_eq!(store.count().await, 2);

    store.remove("doc1").await.unwrap();
    assert_eq!(store.count().await, 1);

    store.remove("doc2").await.unwrap();
    assert_eq!(store.count().await, 0);
}
