# Toolbox - 离线多功能工具箱

<p align="center">
  <img src="public/toolbox.svg" alt="toolbox" width="50" height="50">
</p>

一个基于 `Tauri2`、`Vue3` + `TypeScript` 和 `Rust` 开发的跨平台离线工具箱，提供多种实用工具，满足日常开发和工作需求。

## 项目概述

- **桌面框架**: `Tauri 2`
- **前端**: `Vue 3`、`TypeScript`、`Vite`、`Naive UI`
- **后端**: `Rust`（workspace 多 crate：`base`、`database`、`datafaker`、`monitor`，使用 `sqlx`、`tokio` 及多个 Tauri 插件）
- **数据存储**: `SQLite`（tauri-plugin-sql）
- **特点**: 离线运行、轻量级、跨平台、功能丰富
- **支持平台**: `Windows`、`macOS`、`Linux`

## 功能菜单

### 首页

### 系统监控

- CPU / 内存 / 磁盘 / 电池实时监控
- 进程列表查看与结束进程

### 代码片段

- 代码片段管理与复用

### 待办事项

- 待办事项管理

### 转换

- 文件格式转换: JSON、YAML、TOML 等格式相互转换
- 时间戳: 时间戳与人类可读时间互转
- 进制转换: 二进制、八进制、十进制、十六进制及任意进制互转
- Cron 表达式: 校验表达式并列出接下来 10 次执行时间，支持常用预设与字段生成
- 人民币大小写: 金额数字与中文大写金额互转，支持元/圆、整/正与角分规范写法

### 编码/解码

- Base 编码: Base64 / Base32 / Base58 编码/解码
- URL: URL 编码/解码
- JWT: JWT 解码
- 文本编码: 多字符集编码转换（Unicode / 转义 / C 数组 / 汇编等输出格式）

### 加密

- 对称加密: AES / DES / 3DES / RC4 / Rabbit 加解密

### 格式化

- JSON Editor: JSON 查看、编辑与格式化
- SQL: SQL 格式化
- XML: XML 格式化

### 生成器

- UUID: UUID 生成器（`UUID v4`、`UUID v5`、`UUID v6`、`UUID v7` 等）
- Hash 计算: Hash 生成器（`MD5`、`SHA-1`、`SHA-256`、`SHA-3` 等，支持 HMAC）

### 数据库

- 假数据生成: 可视化节点编排生成表结构假数据（姓名、邮箱、IP、时间、正则等内置生成器）
- 数据库差异: 连接数据库对比表结构差异，生成差异 SQL / 报告 / 建表代码

### 文本

- Markdown: Markdown 预览与编辑
- 文本工具: 字符统计、文本清理（去空行/重复/重音/空白/换行）、排序提取、查找替换、斜线与翻转
- 文本 / JSON 差异: 文本行级差异对比，及 JSON 对象 / 数组结构差异对比

### 随机

- 随机字符串: 随机字符串 / 密码生成（自定义字符集、密码模式）
- 随机数字: 随机整数 / 小数 / 素数 / 十六进制 / 二进制 / 字节生成
- 随机数据: 随机 JSON / XML / CSV / TSV 生成、正则随机数据、文本随机排序

### 网络

- IP 地址转换: IP ↔ 十进制 / 十六进制 / 二进制 / 八进制互转
- 随机 IP / MAC / 时间: 随机 IP（v4/v6）、MAC、时间、日期生成

### 按位计算器

- 按位与 / 或 / 异或 / 非 / NAND / NOR / XNOR / 移位等位运算

### 颜色转换

- 颜色转换: HEX / RGB / HSV / CMYK 颜色互转与取色

### 正则

- 正则可视化: 正则表达式可视化与测试工具

### 图像

- 图片格式转换: 常见图片格式相互转换
- Excalidraw: 白板绘图

### PDF

- 图片转 PDF: 多张图片按顺序合成为一个 PDF
- PDF 合并: 多个 PDF 按列表顺序合并为一个
- PDF 编辑: 删除 / 旋转 / 调整页面顺序
- PDF 添加页码: 在指定位置添加页码或自定义文本
- PDF 拆分: 按页码范围将 PDF 拆分为多个文件

### 其它

- 二维码: 二维码生成（可调尺寸、颜色、纠错级别，支持导出 PNG）
- 剪贴板管理: 剪贴板管理

## 安装与运行

### 前置条件

- 安装 [Node.js](https://nodejs.org/)
- 安装 [Rust](https://www.rust-lang.org/)
- 安装 [Tauri 开发环境](https://tauri.app/zh-cn/start/prerequisites/)

### 开发环境运行

```bash
# 安装依赖
yarn

# 运行开发服务器
cargo tauri dev
```

### 构建生产版本

```bash
# 构建应用
cargo tauri build
```

