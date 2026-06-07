//! G 层黑盒验收测试 — 基于 cucumber-rs
//! 182 个场景，1:1 映射产品文档 (docs/testing/scenarios/catalog.md)

mod world;
mod steps;

use cucumber::World;

#[tokio::main]
async fn main() {
    world::AcceptanceWorld::run("features").await;
}
