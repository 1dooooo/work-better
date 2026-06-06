//! SemanticSearch: 语义搜索引擎
//!
//! 提供高级语义搜索和 RAG 上下文召回功能。

use wb_core::error::Result;

use super::store::VectorStore;
use super::SearchResult;

/// 语义搜索引擎
///
/// 封装 VectorStore，提供更高级的搜索和 RAG 功能。
#[derive(Debug, Clone)]
pub struct SemanticSearch<S: VectorStore> {
    store: S,
}

impl<S: VectorStore> SemanticSearch<S> {
    /// 创建语义搜索引擎
    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// 获取底层 store 的引用
    pub fn store(&self) -> &S {
        &self.store
    }

    /// 语义搜索
    ///
    /// # Arguments
    /// * `query` - 搜索查询
    /// * `top_k` - 返回结果数量
    ///
    /// # Returns
    /// 按相似度降序排列的搜索结果
    pub async fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
        self.store.search(query, top_k).await
    }

    /// 查找相似文档
    ///
    /// # Arguments
    /// * `doc_id` - 参考文档 ID
    /// * `top_k` - 返回结果数量
    pub async fn similar(&self, doc_id: &str, top_k: usize) -> Result<Vec<SearchResult>> {
        self.store.similar(doc_id, top_k).await
    }

    /// RAG 上下文召回
    ///
    /// 根据查询检索相关文档片段，用于增强 LLM 上下文。
    ///
    /// # Arguments
    /// * `query` - 用户查询
    /// * `max_tokens` - 最大 token 数（粗略估计）
    ///
    /// # Returns
    /// 拼接后的上下文字符串
    pub async fn rag_context(&self, query: &str, max_tokens: usize) -> Result<String> {
        self.store.rag_context(query, max_tokens).await
    }

    /// 带阈值的语义搜索
    ///
    /// 只返回相似度高于阈值的结果。
    ///
    /// # Arguments
    /// * `query` - 搜索查询
    /// * `top_k` - 最大返回数量
    /// * `threshold` - 最小相似度阈值（0.0 - 1.0）
    pub async fn search_with_threshold(
        &self,
        query: &str,
        top_k: usize,
        threshold: f32,
    ) -> Result<Vec<SearchResult>> {
        let results = self.store.search(query, top_k).await?;
        Ok(results.into_iter().filter(|r| r.score >= threshold).collect())
    }

    /// 多查询搜索（融合多个查询的结果）
    ///
    /// 对多个查询分别搜索，然后合并去重，按最高相似度排序。
    ///
    /// # Arguments
    /// * `queries` - 查询列表
    /// * `top_k` - 每个查询返回的结果数量
    pub async fn multi_query_search(
        &self,
        queries: &[&str],
        top_k: usize,
    ) -> Result<Vec<SearchResult>> {
        let mut all_results: Vec<SearchResult> = Vec::new();

        for query in queries {
            let results = self.store.search(query, top_k).await?;
            all_results.extend(results);
        }

        // 合并去重（保留最高分数）
        let mut merged: std::collections::HashMap<String, SearchResult> =
            std::collections::HashMap::new();

        for result in all_results {
            let entry = merged
                .entry(result.doc_id.clone())
                .or_insert_with(|| SearchResult {
                    doc_id: result.doc_id.clone(),
                    score: 0.0,
                    content_snippet: result.content_snippet.clone(),
                });

            if result.score > entry.score {
                entry.score = result.score;
            }
        }

        let mut results: Vec<SearchResult> = merged.into_values().collect();
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::embedding::MockEmbedding;
    use crate::vector::store::InMemoryVectorStore;
    use std::sync::Arc;

    async fn create_search_engine() -> SemanticSearch<InMemoryVectorStore> {
        let embedding = Arc::new(MockEmbedding::new(128));
        let store = InMemoryVectorStore::new(embedding);

        store.upsert("doc1", "Rust is a systems programming language").await.unwrap();
        store.upsert("doc2", "Python is great for data science and ML").await.unwrap();
        store.upsert("doc3", "Rust focuses on safety and performance").await.unwrap();
        store.upsert("doc4", "JavaScript is used for web development").await.unwrap();

        SemanticSearch::new(store)
    }

    #[tokio::test]
    async fn test_search() {
        let engine = create_search_engine().await;
        let results = engine.search("programming", 3).await.unwrap();

        assert!(!results.is_empty());
        assert!(results.len() <= 3);

        // 结果应按相似度降序
        for i in 0..results.len() - 1 {
            assert!(results[i].score >= results[i + 1].score);
        }
    }

    #[tokio::test]
    async fn test_similar() {
        let engine = create_search_engine().await;
        let results = engine.similar("doc1", 2).await.unwrap();

        assert_eq!(results.len(), 2);
        // doc1 和 doc3 都关于 Rust，应该相似度较高
        assert!(results.iter().any(|r| r.doc_id == "doc3"));
    }

    #[tokio::test]
    async fn test_rag_context() {
        let engine = create_search_engine().await;
        let context = engine.rag_context("systems programming", 500).await.unwrap();

        assert!(!context.is_empty());
        // 应该包含相关内容
        assert!(context.contains("Rust") || context.contains("programming"));
    }

    #[tokio::test]
    async fn test_search_with_threshold() {
        let engine = create_search_engine().await;
        let results = engine.search_with_threshold("rust", 10, 0.5).await.unwrap();

        // 所有结果的相似度应 >= 0.5
        for result in &results {
            assert!(result.score >= 0.5);
        }
    }

    #[tokio::test]
    async fn test_multi_query_search() {
        let engine = create_search_engine().await;
        let results = engine
            .multi_query_search(&["rust", "performance"], 3)
            .await
            .unwrap();

        // 应该返回去重后的结果
        let doc_ids: Vec<&str> = results.iter().map(|r| r.doc_id.as_str()).collect();
        let unique: std::collections::HashSet<&str> = doc_ids.iter().copied().collect();
        assert_eq!(doc_ids.len(), unique.len());
    }
}
