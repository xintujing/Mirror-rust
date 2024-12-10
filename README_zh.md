# Mirror-rust

[English](README.md) | [中文](README_zh.md)

## 🌟 简介

Mirror-rust 是一个用 Rust 实现的高性能网络库，支持 KCP 协议。它为实时应用程序提供可靠、有序和快速的通信能力。

### ✨ 特性

- 使用 Rust 构建，确保安全性和性能
- 实现 KCP 协议，提供可靠的 UDP 通信
- 线程安全的并发数据结构
- 可配置的网络参数
- 支持实时数据同步
- 精确的十进制计算处理

### 🚀 安装

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
Mirror-rust = { git = "https://github.com/your-username/Mirror-rust.git" }
```

### 📦 依赖项

- Rust 2021 版本
- 主要依赖：
  - kcp2k_rust：KCP 协议实现
  - serde：序列化框架
  - nalgebra：数学计算
  - dashmap：线程安全的并发映射

### 💡 使用方法

请查看 `examples` 目录获取详细的使用示例。

### 🤝 贡献

欢迎贡献代码！请随时提交 Pull Request。

### 📄 许可证

本项目采用 MIT 许可证。
