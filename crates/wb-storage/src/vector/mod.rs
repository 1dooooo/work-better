//! 向量数据库层
//!
//! 提供文档 Embedding、语义搜索和 RAG 上下文召回能力。
//!
//! # 架构
//!
//! - `embedding` - EmbeddingProvider trait 和 MockEmbedding 实现
//! - `store` - VectorStore trait 和 InMemoryVectorStore 实现
//! - `search` - SemanticSearch 高级语义搜索
//! - `sync` - VectorSync 增量同步
//!
//! # 使用示例
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use wb_storage::vector::{
//!     MockEmbedding, InMemoryVectorStore, SemanticSearch, VectorStore,
//! };
//!
//! # async fn example() -> wb_core::error::Result<()> {
//! // 创建 embedding 引擎
//! let embedding = Arc::new(MockEmbedding::new(128));
//!
//! // 创建向量存储
//! let store = InMemoryVectorStore::new(embedding);
//!
//! // 存储文档
//! store.upsert("doc1", "Rust is a systems programming language").await?;
//!
//! // 语义搜索
//! let results = store.search("programming", 5).await?;
//!
//! // RAG 上下文召回
//! let context = store.rag_context("rust performance", 1000).await?;
//! # Ok(())
//! # }
//! ```

pub mod embedding;
pub mod search;
pub mod store;
pub mod sync;

pub use embedding::{EmbeddingProvider, MockEmbedding};
pub use search::SemanticSearch;
pub use store::{InMemoryVectorStore, VectorStore};
pub use sync::{SyncReport, VectorSync};

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// 文档 ID
    pub doc_id: String,
    /// 相似度分数（0.0 - 1.0）
    pub score: f32,
    /// 内容片段（截断到 200 字符）
    pub content_snippet: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_integration() {
        // 创建组件
        let embedding = Arc::new(MockEmbedding::new(128));
        let store = InMemoryVectorStore::new(embedding);

        // 存储文档
        store.upsert("rust", "Rust is a systems programming language").await.unwrap();
        store.upsert("python", "Python is great for data science").await.unwrap();
        store.upsert("js", "JavaScript is used for web development").await.unwrap();

        // 验证存储
        assert_eq!(store.count().await, 3);

        // 语义搜索
        let results = store.search("programming", 2).await.unwrap();
        assert_eq!(results.len(), 2);

        // 相似文档查找
        let similar = store.similar("rust", 2).await.unwrap();
        assert_eq!(similar.len(), 2);

        // RAG 上下文
        let context = store.rag_context("systems", 500).await.unwrap();
        assert!(!context.is_empty());
    }

    #[tokio::test]
    async fn test_semantic_search_integration() {
        let embedding = Arc::new(MockEmbedding::new(128));
        let store = InMemoryVectorStore::new(embedding);

        store.upsert("doc1", "Machine learning algorithms").await.unwrap();
        store.upsert("doc2", "Deep learning neural networks").await.unwrap();
        store.upsert("doc3", "Traditional software engineering").await.unwrap();

        let search = SemanticSearch::new(store);

        // 带阈值的搜索
        let results = search.search_with_threshold("learning", 10, 0.3).await.unwrap();
        for result in &results {
            assert!(result.score >= 0.3);
        }
    }

    #[tokio::test]
    async fn test_sync_integration() {
        let embedding = Arc::new(MockEmbedding::new(128));
        let store = InMemoryVectorStore::new(embedding.clone());
        let sync = VectorSync::new(store.clone());

        // 批量 embedding
        let docs = vec![
            ("doc1", "content 1"),
            ("doc2", "content 2"),
        ];
        let report = sync.batch_reembed(&docs).await.unwrap();
        assert_eq!(report.synced_count, 2);

        // 同步变更
        let report = sync.sync_changed(&["doc1".to_string()]).await.unwrap();
        assert_eq!(report.synced_count, 1);
    }
}
