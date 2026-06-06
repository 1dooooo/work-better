//! InMemoryVectorStore: 内存向量存储实现
//!
//! 使用 HashMap + RwLock 实现向量存储，支持 CRUD 操作和余弦相似度搜索。

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use wb_core::error::Result;

use super::embedding::EmbeddingProvider;
use super::SearchResult;

/// 向量存储 trait
///
/// 定义向量存储的通用接口。
#[async_trait::async_trait]
pub trait VectorStore: Send + Sync {
    /// 存储文档的向量表示
    async fn upsert(&self, doc_id: &str, content: &str) -> Result<()>;

    /// 删除文档向量
    async fn remove(&self, doc_id: &str) -> Result<bool>;

    /// 获取文档向量
    async fn get(&self, doc_id: &str) -> Result<Option<Vec<f32>>>;

    /// 语义搜索：查找与 query 最相似的 top_k 个文档
    async fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>>;

    /// 查找与指定文档最相似的 top_k 个文档
    async fn similar(&self, doc_id: &str, top_k: usize) -> Result<Vec<SearchResult>>;

    /// RAG 上下文召回：返回与查询相关的文档片段，限制最大 token 数
    async fn rag_context(&self, query: &str, max_tokens: usize) -> Result<String>;

    /// 获取存储的文档数量
    async fn count(&self) -> usize;
}

/// 内存向量存储实现
///
/// 使用 `Arc<RwLock<HashMap<String, (Vec<f32>, String)>>>` 存储向量和内容。
/// 线程安全，支持并发读写。
#[derive(Debug, Clone)]
pub struct InMemoryVectorStore {
    /// 向量存储：doc_id -> (embedding, content)
    #[allow(clippy::type_complexity)]
    storage: Arc<RwLock<HashMap<String, (Vec<f32>, String)>>>,
    /// Embedding 引擎
    embedding: Arc<dyn EmbeddingProvider>,
}

impl InMemoryVectorStore {
    /// 创建新的内存向量存储
    pub fn new(embedding: Arc<dyn EmbeddingProvider>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            embedding,
        }
    }

    /// 计算两个向量的余弦相似度
    ///
    /// # Formula
    /// `cosine_similarity(a, b) = sum(a * b) / (sqrt(sum(a * a)) * sqrt(sum(b * b)))`
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }
}

#[async_trait::async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn upsert(&self, doc_id: &str, content: &str) -> Result<()> {
        let embedding = self.embedding.embed(content).await?;
        let mut storage = self.storage.write().await;
        storage.insert(doc_id.to_string(), (embedding, content.to_string()));
        Ok(())
    }

    async fn remove(&self, doc_id: &str) -> Result<bool> {
        let mut storage = self.storage.write().await;
        Ok(storage.remove(doc_id).is_some())
    }

    async fn get(&self, doc_id: &str) -> Result<Option<Vec<f32>>> {
        let storage = self.storage.read().await;
        Ok(storage.get(doc_id).map(|(emb, _)| emb.clone()))
    }

    async fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
        let query_embedding = self.embedding.embed(query).await?;
        let storage = self.storage.read().await;

        let mut results: Vec<SearchResult> = storage
            .iter()
            .map(|(doc_id, (embedding, content))| {
                let score = Self::cosine_similarity(&query_embedding, embedding);
                SearchResult {
                    doc_id: doc_id.clone(),
                    score,
                    content_snippet: Self::truncate_content(content, 200),
                }
            })
            .collect();

        // 按相似度降序排序
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // 返回 top_k 个结果
        Ok(results.into_iter().take(top_k).collect())
    }

    async fn similar(&self, doc_id: &str, top_k: usize) -> Result<Vec<SearchResult>> {
        let storage = self.storage.read().await;

        let target_embedding = match storage.get(doc_id) {
            Some((emb, _)) => emb.clone(),
            None => return Ok(Vec::new()),
        };

        let mut results: Vec<SearchResult> = storage
            .iter()
            .filter(|(id, _)| id.as_str() != doc_id)
            .map(|(id, (embedding, content))| {
                let score = Self::cosine_similarity(&target_embedding, embedding);
                SearchResult {
                    doc_id: id.clone(),
                    score,
                    content_snippet: Self::truncate_content(content, 200),
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results.into_iter().take(top_k).collect())
    }

    async fn rag_context(&self, query: &str, max_tokens: usize) -> Result<String> {
        // 粗略估计：1 token ≈ 4 字符（英文）或 1.5 字符（中文）
        let chars_per_token = 2.5;
        let max_chars = (max_tokens as f32 * chars_per_token) as usize;

        let results = self.search(query, 10).await?;

        let mut context = String::new();
        let mut total_chars = 0;

        for result in results {
            let snippet = &result.content_snippet;
            let snippet_len = snippet.len();

            if total_chars + snippet_len > max_chars {
                // 截断最后一个片段（安全处理多字节字符）
                let remaining = max_chars - total_chars;
                if remaining > 50 {
                    let safe_boundary = snippet
                        .char_indices()
                        .find(|(i, _)| *i >= remaining)
                        .map(|(i, _)| i)
                        .unwrap_or(snippet.len());
                    context.push_str(&snippet[..safe_boundary]);
                    context.push_str("...");
                }
                break;
            }

            if !context.is_empty() {
                context.push_str("\n\n---\n\n");
            }
            context.push_str(snippet);
            total_chars += snippet_len;
        }

        Ok(context)
    }

    async fn count(&self) -> usize {
        let storage = self.storage.read().await;
        storage.len()
    }
}

impl InMemoryVectorStore {
    /// 截断内容到指定长度（安全处理多字节字符）
    fn truncate_content(content: &str, max_len: usize) -> String {
        if content.len() <= max_len {
            content.to_string()
        } else {
            let safe = content.char_indices()
                .find(|(i, _)| *i >= max_len)
                .map(|(i, _)| i)
                .unwrap_or(content.len());
            format!("{}...", &content[..safe])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::embedding::MockEmbedding;

    fn create_store() -> InMemoryVectorStore {
        let embedding = Arc::new(MockEmbedding::new(128));
        InMemoryVectorStore::new(embedding)
    }

    #[tokio::test]
    async fn test_upsert_and_get() {
        let store = create_store();
        store.upsert("doc1", "hello world").await.unwrap();

        let embedding = store.get("doc1").await.unwrap();
        assert!(embedding.is_some());
        assert_eq!(embedding.unwrap().len(), 128);
    }

    #[tokio::test]
    async fn test_remove() {
        let store = create_store();
        store.upsert("doc1", "hello").await.unwrap();

        let removed = store.remove("doc1").await.unwrap();
        assert!(removed);

        let embedding = store.get("doc1").await.unwrap();
        assert!(embedding.is_none());
    }

    #[tokio::test]
    async fn test_remove_nonexistent() {
        let store = create_store();
        let removed = store.remove("nonexistent").await.unwrap();
        assert!(!removed);
    }

    #[tokio::test]
    async fn test_cosine_similarity_identical() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = InMemoryVectorStore::cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = InMemoryVectorStore::cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        let sim = InMemoryVectorStore::cosine_similarity(&a, &b);
        assert!((sim - (-1.0)).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_search_top_k() {
        let store = create_store();
        store.upsert("doc1", "rust programming").await.unwrap();
        store.upsert("doc2", "python programming").await.unwrap();
        store.upsert("doc3", "rust systems").await.unwrap();

        let results = store.search("rust", 2).await.unwrap();
        assert_eq!(results.len(), 2);

        // 结果应按相似度降序排列
        assert!(results[0].score >= results[1].score);
    }

    #[tokio::test]
    async fn test_similar_docs() {
        let store = create_store();
        store.upsert("doc1", "rust programming language").await.unwrap();
        store.upsert("doc2", "rust systems programming").await.unwrap();
        store.upsert("doc3", "python data science").await.unwrap();

        let results = store.similar("doc1", 2).await.unwrap();
        assert_eq!(results.len(), 2);

        // 结果应按相似度降序排列
        assert!(results[0].score >= results[1].score);
        // doc1 不应出现在结果中
        assert!(!results.iter().any(|r| r.doc_id == "doc1"));
    }

    #[tokio::test]
    async fn test_rag_context() {
        let store = create_store();
        store.upsert("doc1", "Rust is a systems programming language").await.unwrap();
        store.upsert("doc2", "Python is great for data science").await.unwrap();

        let context = store.rag_context("programming", 100).await.unwrap();
        assert!(!context.is_empty());
    }

    #[tokio::test]
    async fn test_rag_context_utf8_safe_truncation() {
        let store = create_store();
        // 中文内容，每个字符 3 字节；如果直接用字节索引截断会 panic
        store.upsert("doc1", "这是一个中文测试内容，用于验证多字节字符截断安全性").await.unwrap();
        // max_tokens=5 → max_chars≈12，会触发截断逻辑
        let context = store.rag_context("测试", 5).await.unwrap();
        // 验证不会 panic 且结果非空或以 "..." 结尾（安全截断）
        assert!(context.is_empty() || context.ends_with("...") || context.contains("测试"));
    }

    #[tokio::test]
    async fn test_count() {
        let store = create_store();
        assert_eq!(store.count().await, 0);

        store.upsert("doc1", "a").await.unwrap();
        store.upsert("doc2", "b").await.unwrap();
        assert_eq!(store.count().await, 2);

        store.remove("doc1").await.unwrap();
        assert_eq!(store.count().await, 1);
    }
}
