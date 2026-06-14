//! G 层黑盒验收测试 — 基于 cucumber-rs
//! 182 个场景，1:1 映射产品文档 (docs/testing/scenarios/catalog.md)

mod world;
mod steps;

/// 测试公共模块（wiremock 基础设施等）
/// 通过 #[path] 引入项目级 tests/common/ 下的模块
#[path = "../../common/mod.rs"]
pub mod common;

use cucumber::World;

#[tokio::main]
async fn main() {
    world::AcceptanceWorld::run("tests/acceptance/features").await;
}
