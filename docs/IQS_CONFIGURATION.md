# 阿里云 IQS 搜索配置指南

## 概述

CoSurf 现已集成阿里云智能查询服务 (Intelligent Query Service, IQS) 作为 Web Search 工具的后端。IQS 提供实时网页搜索、内容提取和多搜索引擎支持等功能。

## 获取 API Key

### 步骤 1: 登录阿里云控制台

访问 [阿里云 IQS 控制台](https://help.aliyun.com/zh/document_detail/3025781.html)

### 步骤 2: 创建 API Key

1. 登录阿里云账号
2. 进入 IQS 服务页面
3. 点击"创建 API Key"
4. 复制生成的 `ALIYUN_IQS_API_KEY`

## 在 CoSurf 中配置

### 方法一: 通过设置界面（推荐）

1. 打开 CoSurf
2. 点击左下角设置图标 ⚙️
3. 选择 **工具** 标签页
4. 在"阿里云 IQS 搜索配置"区域：
   - 粘贴您的 `ALIYUN_IQS_API_KEY`
   - 点击 **保存** 按钮
5. 看到绿色 ✓ "API Key 已配置"提示即表示成功

### 方法二: 直接编辑数据库

如果您熟悉 SQLite，可以直接编辑数据库文件：

```sql
UPDATE settings SET value = 'your-api-key-here' WHERE key = 'iqsApiKey';
```

数据库位置：`%APPDATA%\CoSurf\cosurf.db`

## 使用 IQS 搜索

### 基本用法

当您需要搜索信息时，可以直接告诉 AI：

```
用户: 帮我搜索最新的 AI 技术新闻
AI: [调用 web_search 工具]
    参数: {
      "query": "AI 技术新闻",
      "engine_type": "News",
      "time_range": "OneDay",
      "max_results": 5
    }
```

### 高级参数

#### 1. engine_type (搜索引擎类型)

- **Generic** (默认) - 通用搜索
- **News** - 新闻搜索
- **Academic** - 学术搜索

示例：
```
用户: 搜索关于量子计算的学术论文
AI 应该使用: engine_type = "Academic"
```

#### 2. time_range (时间范围)

- **OneDay** - 过去一天
- **OneWeek** (默认) - 过去一周
- **OneMonth** - 过去一个月
- **OneYear** - 过去一年
- **NoLimit** - 不限时间

示例：
```
用户: 今天的热点新闻有哪些？
AI 应该使用: time_range = "OneDay"
```

#### 3. max_results (最大结果数)

- 范围: 1-20
- 默认: 5

示例：
```
用户: 给我找10个关于机器学习的资源
AI 应该使用: max_results = 10
```

## 示例对话

### 示例 1: 搜索新闻

```
用户: 今天有什么科技新闻？

AI: 让我为您搜索今天的科技新闻...
[调用 web_search]
  query: "科技新闻"
  engine_type: "News"
  time_range: "OneDay"
  max_results: 5

AI: 找到 5 个今日科技新闻：

[1] OpenAI 发布新模型 GPT-5
URL: https://example.com/news1
摘要: OpenAI 今天发布了新一代语言模型...

[2] 苹果推出新款 MacBook Pro
URL: https://example.com/news2
摘要: 苹果公司今天推出了搭载 M4 芯片的...

...
```

### 示例 2: 学术研究

```
用户: 查找关于深度学习在医疗领域应用的最新研究

AI: 我来搜索相关的学术研究...
[调用 web_search]
  query: "深度学习 医疗 应用"
  engine_type: "Academic"
  time_range: "OneMonth"
  max_results: 10

AI: 找到 10 篇相关学术论文：
...
```

### 示例 3: 实时信息

```
用户: 现在的比特币价格是多少？

AI: 让我查询最新的比特币价格...
[调用 web_search]
  query: "比特币价格 BTC USD"
  engine_type: "Generic"
  time_range: "OneDay"
  max_results: 3

AI: 根据最新数据：
...
```

## 故障排除

### 问题 1: "未配置阿里云 IQS API Key"

**症状**: AI 返回错误消息说未配置 API Key

**解决**:
1. 检查设置 → 工具中是否已配置 API Key
2. 确认 API Key 是否正确（没有多余空格）
3. 重启 CoSurf 应用

### 问题 2: "IQS API 请求失败"

**症状**: 显示 HTTP 错误码

**可能原因**:
- **401 Unauthorized**: API Key 无效或过期
- **403 Forbidden**: API 权限不足
- **429 Too Many Requests**: 超出配额限制
- **500 Internal Server Error**: IQS 服务暂时不可用

**解决**:
1. 检查 API Key 是否有效
2. 登录阿里云控制台查看配额使用情况
3. 稍后重试

### 问题 3: "未找到相关搜索结果"

**症状**: 搜索返回空结果

**可能原因**:
- 查询词太具体或太少
- 时间范围设置过窄
- 网络问题

**解决**:
1. 尝试更通用的查询词
2. 扩大时间范围
3. 检查网络连接

### 问题 4: 搜索结果不准确

**症状**: 返回的结果与查询不相关

**解决**:
1. 优化查询词，使用更精确的关键词
2. 选择合适的 engine_type
3. 调整 time_range 到合适的时间段

## 最佳实践

### 1. 选择合适的搜索引擎类型

- 搜索新闻 → 使用 `News`
- 搜索学术论文 → 使用 `Academic`
- 其他情况 → 使用 `Generic`

### 2. 合理设置时间范围

- 实时信息（价格、天气等）→ `OneDay`
- 近期事件 → `OneWeek`
- 历史研究 → `OneMonth` 或更长

### 3. 控制结果数量

- 快速浏览 → 3-5 条
- 深入研究 → 10-15 条
- 避免过多（>15），影响性能

### 4. 优化查询词

✅ 好的查询词：
- "2024年人工智能发展趋势"
- "Python 机器学习教程"
- "气候变化最新研究"

❌ 不好的查询词：
- "AI"（太宽泛）
- "那个东西怎么用"（不明确）

## 计费说明

IQS 服务按调用次数计费，具体价格请参考 [阿里云官方定价](https://www.aliyun.com/price)。

建议：
- 开发测试时使用少量调用
- 生产环境注意监控用量
- 设置预算告警防止超额

## 隐私与安全

- API Key 存储在本地数据库中
- 不会上传到云端（除 IQS API 调用外）
- 建议定期轮换 API Key
- 不要与他人共享 API Key

## 相关链接

- [阿里云 IQS 官方文档](https://help.aliyun.com/zh/document_detail/3025781.html)
- [IQS API 参考](https://help.aliyun.com/zh/iqs/developer-reference/api-overview)
- [CoSurf Skills 系统](./SKILLS_GUIDE.md)

---

如有问题，请提交 Issue 或联系技术支持。
