//! EmbeddingEngine: 文本向量化引擎
//!
//! 提供文本到向量的转换能力。当前实现为 MockEmbedding（随机向量），
//! 预留 trait 接口供后续替换为 OpenAI 等真实 embedding 服务。

use async_trait::async_trait;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use wb_core::error::Result;

/// Embedding 引擎 trait
///
/// 定义文本向量化的通用接口，支持不同实现（Mock、OpenAI 等）。
#[async_trait]
pub trait EmbeddingProvider: Send + Sync + std::fmt::Debug {
    /// 将文本转换为向量
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// 获取向量维度
    fn dimensions(&self) -> usize;
}

/// Mock Embedding 引擎
///
/// 使用确定性哈希生成伪随机向量，用于测试和开发。
/// 相同文本总是生成相同的向量。
#[derive(Debug, Clone)]
pub struct MockEmbedding {
    dimensions: usize,
}

impl MockEmbedding {
    /// 创建 MockEmbedding 实例
    ///
    /// # Arguments
    /// * `dimensions` - 向量维度（默认 128）
    pub fn new(dimensions: usize) -> Self {
        Self { dimensions }
    }

    /// 创建默认维度（128）的 MockEmbedding
    pub fn default_128() -> Self {
        Self::new(128)
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // 使用文本的哈希值作为种子，生成确定性向量
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let seed = hasher.finish();

        let mut vector = Vec::with_capacity(self.dimensions);

        // 生成伪随机向量（确定性）
        for i in 0..self.dimensions {
            let val = ((seed.wrapping_add(i as u64)).wrapping_mul(6364136223846793005) >> 33) as f32
                / (1u64 << 31) as f32;
            vector.push(val - 1.0); // 归一化到 [-1, 1]
        }

        // L2 归一化
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut vector {
                *val /= norm;
            }
        }

        Ok(vector)
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_embedding_dimensions() {
        let engine = MockEmbedding::new(64);
        let vector = engine.embed("test text").await.unwrap();
        assert_eq!(vector.len(), 64);
    }

    #[tokio::test]
    async fn test_mock_embedding_deterministic() {
        let engine = MockEmbedding::default_128();
        let vec1 = engine.embed("hello world").await.unwrap();
        let vec2 = engine.embed("hello world").await.unwrap();
        assert_eq!(vec1, vec2);
    }

    #[tokio::test]
    async fn test_mock_embedding_different_texts() {
        let engine = MockEmbedding::default_128();
        let vec1 = engine.embed("hello").await.unwrap();
        let vec2 = engine.embed("world").await.unwrap();
        // 不同文本应生成不同向量
        assert_ne!(vec1, vec2);
    }

    #[tokio::test]
    async fn test_mock_embedding_normalized() {
        let engine = MockEmbedding::new(128);
        let vector = engine.embed("normalize test").await.unwrap();
        // 验证 L2 归一化（向量长度约为 1.0）
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }
}
