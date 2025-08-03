# Toolbox - 离线多功能工具箱

<p align="center">
  <img src="public/toolbox.svg" alt="toolbox" width="50" height="50">
</p>

一个基于`Tauri2`、`Vue3` + `JavaScript`和`Rust`开发的跨平台离线工具箱，提供多种实用工具，满足日常开发和工作需求。

## 项目概述

- **技术栈**: `Tauri2`、`Vue3`、`JavaScript`、`Rust`
- **特点**: 离线运行、轻量级、跨平台、功能丰富
- **支持平台**: `Windows`、`macOS`、`Linux`

## 核心功能

### 1. 转换工具

- 进制转换: 二进制、八进制、十进制、十六进制互转
- 格式转换: JSON、YAML、TOML 格式相互转换
- 时间转换: 时间戳与人类可读时间互转
- [ ] Cron 表达式解析与生成

### 2. 编码/解码工具

- URL 编码/解码
- Base64 文本编码/解码
- [ ] Base64 图片编码/解码
- [ ] JWT 解码

### 3. 格式化工具

- SQL 格式化
- XML 格式化
- TOML 格式化

### 4. 生成器工具

- UUID 生成器（`UUID v4`、`UUID v5`、`UUID v6`、`UUID v7`等）
- Hash 生成器（`MD5`、`SHA-1`、`SHA-256` 等）
- 文件校验和计算器
- [ ] 数据模拟生成器

### 5. 文本工具

- [ ] 正则表达式可视化与测试工具
- Markdown 预览




## 安装与运行

### 前置条件

- 安装 [Node.js](https://nodejs.org/)
- 安装 [Rust](https://www.rust-lang.org/)
- 安装 [Tauri 开发环境](https://tauri.app/zh-cn/start/prerequisites/)

### 开发环境运行

```bash
# 安装依赖
npm install

# 运行开发服务器
npm run tauri dev
```

### 构建生产版本

```bash
# 构建应用
npm run tauri build
```
