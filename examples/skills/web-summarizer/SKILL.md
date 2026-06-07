---
name: 网页内容总结
description: 打开指定网页并提取、总结其主要内容，支持多语言翻译
tags: [summary, web, translate]
---

# 网页内容总结

这是一个综合工具，用于打开网页、提取内容并生成结构化总结。

## 使用场景

- 用户要求总结某个网页的内容
- 用户想了解某个 URL 的主要信息
- 用户需要将外文网页翻译并总结

## 执行步骤

1. **打开网页**：使用 `open_url` 工具打开目标网页
2. **提取内容**：使用 `summarize_page` 工具获取页面摘要
3. **翻译（可选）**：如果用户要求翻译，使用 `translate` 工具
4. **生成回答**：将提取的内容整理为结构化的总结

## 可用工具

- **open_url**: 打开指定 URL
  ```
  参数: { "url": "https://example.com" }
  ```
- **summarize_page**: 总结当前页面
  ```
  参数: { "max_length": 500 }  # 可选，最大摘要长度
  ```
- **translate**: 翻译页面内容
  ```
  参数: { "target_language": "zh" }  # zh/en/ja 等
  ```
- **export_markdown**: 导出为 Markdown
  ```
  参数: {}  # 无需参数
  ```

## 执行流程示例

用户说："帮我总结一下 https://example.com/article 这篇文章"

1. 调用 `open_url(url="https://example.com/article")`
2. 等待页面加载完成
3. 调用 `summarize_page(max_length=800)`
4. 将摘要内容以结构化方式返回给用户

## 注意事项

- 某些网页可能需要较长时间加载，请耐心等待
- 如果页面内容为空，可能是反爬虫限制
- 总结长度建议控制在 500-1000 字符之间
