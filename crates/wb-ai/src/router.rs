//! ModelRouter: 根据任务类型和置信度决定是否升级到大模型

use serde::{Deserialize, Serialize};

/// 任务类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskType {
    EntityExtraction,
    TaskIdentification,
    Summarization,
    SentimentAnalysis,
    RelationAnalysis,
    PatternRecognition,
    Classification,
}

/// 升级阈值配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeThreshold {
    pub task_type: TaskType,
    /// 置信度低于此值时升级到大模型
    pub threshold: f64,
    /// 是否强制使用大模型（忽略置信度）
    pub force_large_model: bool,
}

impl Default for UpgradeThreshold {
    fn default() -> Self {
        // 这个默认值不会直接使用，仅为满足 Default trait
        Self {
            task_type: TaskType::Classification,
            threshold: 0.6,
            force_large_model: false,
        }
    }
}

/// 模型路由器：根据任务类型和置信度决定使用小模型还是大模型
pub struct ModelRouter {
    thresholds: Vec<UpgradeThreshold>,
}

impl ModelRouter {
    /// 使用默认阈值创建路由器
    pub fn new() -> Self {
        Self {
            thresholds: Self::default_thresholds(),
        }
    }

    /// 使用自定义阈值创建路由器
    pub fn with_thresholds(thresholds: Vec<UpgradeThreshold>) -> Self {
        Self { thresholds }
    }

    /// 判断是否应升级到大模型
    ///
    /// 返回 `true` 表示应使用大模型（置信度低于阈值，或任务强制使用大模型）
    pub fn should_upgrade(&self, task_type: &TaskType, confidence: f64) -> bool {
        match self.get_threshold(task_type) {
            Some(threshold) => {
                if threshold.force_large_model {
                    return true;
                }
                confidence < threshold.threshold
            }
            // 未配置阈值的任务类型，不升级
            None => false,
        }
    }

    /// 获取指定任务类型的阈值配置
    pub fn get_threshold(&self, task_type: &TaskType) -> Option<&UpgradeThreshold> {
        self.thresholds
            .iter()
            .find(|t| &t.task_type == task_type)
    }

    /// 默认阈值配置
    fn default_thresholds() -> Vec<UpgradeThreshold> {
        vec![
            UpgradeThreshold {
                task_type: TaskType::EntityExtraction,
                threshold: 0.7,
                force_large_model: false,
            },
            UpgradeThreshold {
                task_type: TaskType::TaskIdentification,
                threshold: 0.6,
                force_large_model: false,
            },
            UpgradeThreshold {
                task_type: TaskType::Summarization,
                threshold: 0.6,
                force_large_model: false,
            },
            UpgradeThreshold {
                task_type: TaskType::SentimentAnalysis,
                threshold: 0.8,
                force_large_model: false,
            },
            UpgradeThreshold {
                task_type: TaskType::RelationAnalysis,
                threshold: 0.7,
                force_large_model: false,
            },
            UpgradeThreshold {
                task_type: TaskType::PatternRecognition,
                threshold: 0.0,
                force_large_model: true,
            },
            UpgradeThreshold {
                task_type: TaskType::Classification,
                threshold: 0.6,
                force_large_model: false,
            },
        ]
    }
}

impl Default for ModelRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_extraction_below_threshold() {
        let router = ModelRouter::new();
        assert!(router.should_upgrade(&TaskType::EntityExtraction, 0.5));
    }

    #[test]
    fn test_entity_extraction_above_threshold() {
        let router = ModelRouter::new();
        assert!(!router.should_upgrade(&TaskType::EntityExtraction, 0.9));
    }

    #[test]
    fn test_entity_extraction_at_threshold() {
        let router = ModelRouter::new();
        // 置信度等于阈值时不升级（只在 < 时升级）
        assert!(!router.should_upgrade(&TaskType::EntityExtraction, 0.7));
    }

    #[test]
    fn test_task_identification_below_threshold() {
        let router = ModelRouter::new();
        assert!(router.should_upgrade(&TaskType::TaskIdentification, 0.3));
    }

    #[test]
    fn test_task_identification_above_threshold() {
        let router = ModelRouter::new();
        assert!(!router.should_upgrade(&TaskType::TaskIdentification, 0.8));
    }

    #[test]
    fn test_summarization_below_threshold() {
        let router = ModelRouter::new();
        assert!(router.should_upgrade(&TaskType::Summarization, 0.4));
    }

    #[test]
    fn test_summarization_above_threshold() {
        let router = ModelRouter::new();
        assert!(!router.should_upgrade(&TaskType::Summarization, 0.9));
    }

    #[test]
    fn test_sentiment_analysis_below_threshold() {
        let router = ModelRouter::new();
        assert!(router.should_upgrade(&TaskType::SentimentAnalysis, 0.6));
    }

    #[test]
    fn test_sentiment_analysis_above_threshold() {
        let router = ModelRouter::new();
        assert!(!router.should_upgrade(&TaskType::SentimentAnalysis, 0.9));
    }

    #[test]
    fn test_relation_analysis_below_threshold() {
        let router = ModelRouter::new();
        assert!(router.should_upgrade(&TaskType::RelationAnalysis, 0.5));
    }

    #[test]
    fn test_relation_analysis_above_threshold() {
        let router = ModelRouter::new();
        assert!(!router.should_upgrade(&TaskType::RelationAnalysis, 0.8));
    }

    #[test]
    fn test_pattern_recognition_always_forced() {
        let router = ModelRouter::new();
        // PatternRecognition 强制使用大模型，无论置信度
        assert!(router.should_upgrade(&TaskType::PatternRecognition, 0.0));
        assert!(router.should_upgrade(&TaskType::PatternRecognition, 0.5));
        assert!(router.should_upgrade(&TaskType::PatternRecognition, 0.99));
        assert!(router.should_upgrade(&TaskType::PatternRecognition, 1.0));
    }

    #[test]
    fn test_classification_below_threshold() {
        let router = ModelRouter::new();
        assert!(router.should_upgrade(&TaskType::Classification, 0.4));
    }

    #[test]
    fn test_classification_above_threshold() {
        let router = ModelRouter::new();
        assert!(!router.should_upgrade(&TaskType::Classification, 0.8));
    }

    #[test]
    fn test_custom_thresholds() {
        let thresholds = vec![UpgradeThreshold {
            task_type: TaskType::Classification,
            threshold: 0.9,
            force_large_model: false,
        }];
        let router = ModelRouter::with_thresholds(thresholds);
        // 0.8 < 0.9，应升级
        assert!(router.should_upgrade(&TaskType::Classification, 0.8));
        // 0.95 >= 0.9，不升级
        assert!(!router.should_upgrade(&TaskType::Classification, 0.95));
    }

    #[test]
    fn test_unknown_task_type_no_upgrade() {
        // 使用自定义阈值，只包含 Classification
        let thresholds = vec![UpgradeThreshold {
            task_type: TaskType::Classification,
            threshold: 0.6,
            force_large_model: false,
        }];
        let router = ModelRouter::with_thresholds(thresholds);
        // EntityExtraction 未配置，不升级
        assert!(!router.should_upgrade(&TaskType::EntityExtraction, 0.1));
    }

    #[test]
    fn test_get_threshold() {
        let router = ModelRouter::new();
        let threshold = router.get_threshold(&TaskType::SentimentAnalysis);
        assert!(threshold.is_some());
        let t = threshold.unwrap();
        assert_eq!(t.threshold, 0.8);
        assert!(!t.force_large_model);
    }

    #[test]
    fn test_get_threshold_forced() {
        let router = ModelRouter::new();
        let threshold = router.get_threshold(&TaskType::PatternRecognition);
        assert!(threshold.is_some());
        assert!(threshold.unwrap().force_large_model);
    }
}
