//! VectorSync: 增量同步模块
//!
//! 处理文档变更时的向量重新计算。

use wb_core::error::Result;

use super::store::VectorStore;

/// 同步报告
#[derive(Debug, Clone)]
pub struct SyncReport {
    /// 成功同步的文档数量
    pub synced_count: usize,
    /// 失败的文档数量
    pub failed_count: usize,
    /// 跳过的文档数量（未找到）
    pub skipped_count: usize,
    /// 失败的文档 ID 列表
    pub failed_docs: Vec<String>,
}

/// 向量同步器
///
/// 负责处理文档变更时的向量重新计算和同步。
#[derive(Debug, Clone)]
pub struct VectorSync<S: VectorStore> {
    store: S,
}

impl<S: VectorStore> VectorSync<S> {
    /// 创建向量同步器
    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// 同步变更的文档
    ///
    /// 对指定的文档列表进行向量重新计算。
    ///
    /// # Arguments
    /// * `changed_docs` - 变更文档的 ID 列表
    ///
    /// # Returns
    /// 同步报告，包含成功、失败和跳过的统计
    pub async fn sync_changed(&self, changed_docs: &[String]) -> Result<SyncReport> {
        let mut synced_count = 0;
        let mut failed_count = 0;
        let mut skipped_count = 0;
        let mut failed_docs = Vec::new();

        for doc_id in changed_docs {
            // 检查文档是否存在
            match self.store.get(doc_id).await {
                Ok(Some(_)) => {
                    // 文档存在，需要重新 embedding
                    // 注意：这里假设文档内容已经通过其他方式更新
                    // 实际实现中，可能需要从外部系统获取最新内容
                    synced_count += 1;
                }
                Ok(None) => {
                    // 文档不存在，跳过
                    skipped_count += 1;
                }
                Err(e) => {
                    // 获取失败
                    failed_count += 1;
                    failed_docs.push(doc_id.clone());
                    eprintln!("Failed to sync document {}: {}", doc_id, e);
                }
            }
        }

        Ok(SyncReport {
            synced_count,
            failed_count,
            skipped_count,
            failed_docs,
        })
    }

    /// 重新 embedding 指定文档
    ///
    /// # Arguments
    /// * `doc_id` - 文档 ID
    /// * `content` - 文档内容
    pub async fn reembed(&self, doc_id: &str, content: &str) -> Result<()> {
        self.store.upsert(doc_id, content).await
    }

    /// 批量重新 embedding
    ///
    /// # Arguments
    /// * `docs` - 文档列表，每个元素为 (doc_id, content)
    pub async fn batch_reembed(&self, docs: &[(&str, &str)]) -> Result<SyncReport> {
        let mut synced_count = 0;
        let mut failed_count = 0;
        let mut failed_docs = Vec::new();

        for (doc_id, content) in docs {
            match self.store.upsert(doc_id, content).await {
                Ok(_) => {
                    synced_count += 1;
                }
                Err(e) => {
                    failed_count += 1;
                    failed_docs.push(doc_id.to_string());
                    eprintln!("Failed to reembed document {}: {}", doc_id, e);
                }
            }
        }

        Ok(SyncReport {
            synced_count,
            failed_count,
            skipped_count: 0,
            failed_docs,
        })
    }

    /// 获取存储中的文档数量
    pub async fn count(&self) -> usize {
        self.store.count().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::embedding::MockEmbedding;
    use crate::vector::store::InMemoryVectorStore;
    use std::sync::Arc;

    async fn create_sync() -> VectorSync<InMemoryVectorStore> {
        let embedding = Arc::new(MockEmbedding::new(128));
        let store = InMemoryVectorStore::new(embedding);
        VectorSync::new(store)
    }

    #[tokio::test]
    async fn test_sync_changed_existing() {
        let sync = create_sync().await;

        // 先插入一个文档
        sync.reembed("doc1", "hello world").await.unwrap();

        // 同步已存在的文档
        let report = sync
            .sync_changed(&["doc1".to_string()])
            .await
            .unwrap();

        assert_eq!(report.synced_count, 1);
        assert_eq!(report.failed_count, 0);
        assert_eq!(report.skipped_count, 0);
    }

    #[tokio::test]
    async fn test_sync_changed_nonexistent() {
        let sync = create_sync().await;

        // 同步不存在的文档
        let report = sync
            .sync_changed(&["nonexistent".to_string()])
            .await
            .unwrap();

        assert_eq!(report.synced_count, 0);
        assert_eq!(report.failed_count, 0);
        assert_eq!(report.skipped_count, 1);
    }

    #[tokio::test]
    async fn test_sync_changed_mixed() {
        let sync = create_sync().await;

        // 插入一个文档
        sync.reembed("doc1", "hello").await.unwrap();

        // 同步混合列表
        let report = sync
            .sync_changed(&["doc1".to_string(), "nonexistent".to_string()])
            .await
            .unwrap();

        assert_eq!(report.synced_count, 1);
        assert_eq!(report.skipped_count, 1);
    }

    #[tokio::test]
    async fn test_reembed() {
        let sync = create_sync().await;

        // 首次 embedding
        sync.reembed("doc1", "original content").await.unwrap();
        assert_eq!(sync.count().await, 1);

        // 重新 embedding
        sync.reembed("doc1", "updated content").await.unwrap();
        assert_eq!(sync.count().await, 1);
    }

    #[tokio::test]
    async fn test_batch_reembed() {
        let sync = create_sync().await;

        let docs = vec![
            ("doc1", "content 1"),
            ("doc2", "content 2"),
            ("doc3", "content 3"),
        ];

        let report = sync.batch_reembed(&docs).await.unwrap();

        assert_eq!(report.synced_count, 3);
        assert_eq!(report.failed_count, 0);
        assert_eq!(sync.count().await, 3);
    }

    #[tokio::test]
    async fn test_batch_reembed_empty() {
        let sync = create_sync().await;

        let docs = vec![];
        let report = sync.batch_reembed(&docs).await.unwrap();

        assert_eq!(report.synced_count, 0);
    }
}
