---
name: 阿里云 IQS 搜索
description: 使用阿里云 IQS 服务进行实时网页搜索，支持多种搜索引擎和时间范围过滤
tags: [search, web, iqs]
---

# 阿里云 IQS 搜索

这是一个网页搜索工具，使用阿里云 IQS (Intelligent Query Service) 进行实时信息检索。

## 使用场景

- 用户需要查找最新的互联网信息
- 用户需要验证某个事实或数据
- 用户需要获取特定主题的多个信息来源

## 执行步骤

1. **分析用户意图**：从用户消息中提取搜索关键词
2. **调用搜索工具**：使用 `web_search` 工具执行搜索
3. **处理结果**：解析搜索结果，提取关键信息
4. **总结回答**：将搜索结果整理为清晰的回答

## 参数说明

搜索时考虑以下参数：

- **query**: 搜索关键词，从用户问题中提取核心内容
- **engine_type**: 搜索引擎类型
  - `Generic`: 通用搜索（默认）
  - `News`: 新闻搜索
  - `Academic`: 学术搜索
- **time_range**: 时间范围
  - `OneDay`: 一天内
  - `OneWeek`: 一周内（默认）
  - `OneMonth`: 一个月内
  - `OneYear`: 一年内
  - `NoLimit`: 不限制
- **max_results**: 最大返回结果数（1-20，默认 5）

## 示例

用户问："最近有什么关于 AI 的新闻？"

执行步骤：
1. 调用 `web_search(query="AI 人工智能 最新进展", engine_type="News", time_range="OneWeek", max_results=5)`
2. 解析返回的搜索结果
3. 整理为简洁的新闻摘要列表
