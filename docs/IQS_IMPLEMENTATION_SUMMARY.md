# Web Search 工具切换到阿里云 IQS 实现总结

## 概述

已将 CoSurf 的 Web Search 工具从通用搜索引擎切换到阿里云智能查询服务 (IQS)，提供更强大、更准确的实时搜索能力。

## 实现内容

### ✅ 1. 前端配置界面

**文件**: `src-web/src/components/settings/SettingsPage.tsx`

- 在"工具"设置页面添加 IQS API Key 配置区域
- 提供密码输入框（隐藏 API Key）
- 保存按钮和状态提示
- 链接到阿里云文档获取 API Key

**功能**:
```typescript
- 输入 ALIYUN_IQS_API_KEY
- 保存到 settings store
- 显示配置状态（已配置/未配置）
```

### ✅ 2. 类型定义更新

**文件**: `packages/shared/src/settings.ts`

添加了 `iqsApiKey` 字段到 `AppSettings` 接口：

```typescript
export interface AppSettings {
  // ... 其他字段
  iqsApiKey: string; // 阿里云 IQS API Key
}
```

### ✅ 3. 后端工具执行

**文件**: `src-tauri/src/ai/stream.rs`

在 `execute_tool` 函数中实现了完整的 IQS API 调用逻辑：

```rust
"web_search" => {
    // 1. 提取参数 (query, engine_type, time_range, max_results)
    // 2. 从数据库获取 IQS API Key
    // 3. 验证 API Key 是否存在
    // 4. 构建 HTTP 请求
    // 5. 调用 IQS API
    // 6. 解析响应
    // 7. 格式化输出结果
}
```

**特性**:
- 参数验证和默认值
- API Key 安全检查
- 错误处理和友好提示
- 详细的日志记录
- 30秒超时控制

### ✅ 4. 工具 Schema 更新

**文件**: `src-tauri/src/ai/tools.rs`

更新了 `web_search` 工具的参数定义：

```json
{
  "query": { "type": "string", "description": "搜索查询词" },
  "engine_type": { 
    "type": "string",
    "enum": ["Generic", "News", "Academic"],
    "default": "Generic"
  },
  "time_range": {
    "type": "string",
    "enum": ["OneDay", "OneWeek", "OneMonth", "OneYear", "NoLimit"],
    "default": "OneWeek"
  },
  "max_results": {
    "type": "integer",
    "minimum": 1,
    "maximum": 20,
    "default": 5
  }
}
```

### ✅ 5. 系统提示词更新

**文件**: `src-tauri/src/commands/ai.rs`

更新了 AI 的系统提示词，说明 web_search 工具的使用方法和配置要求：

```
6. **web_search** - 联网搜索（使用阿里云 IQS）
   - 用途：获取最新信息、实时数据、新闻热点
   - 参数：query, engine_type, time_range, max_results
   - 注意：需要在设置中配置 ALIYUN_IQS_API_KEY 才能使用
```

### ✅ 6. 示例 Skill

**文件**: `examples/alibabacloud-iqs-search-skill.json`

创建了 IQS Search Skill 示例，展示如何使用 Node.js 脚本调用 IQS API：

- 支持环境变量和配置文件读取 API Key
- 完整的 HTTP 请求实现
- 结果格式化处理
- 错误处理

### ✅ 7. 文档

创建了完整的配置和使用文档：

**文件**: `docs/IQS_CONFIGURATION.md`

包含：
- API Key 获取步骤
- 配置方法（UI / 数据库）
- 使用示例（基本 / 高级）
- 参数说明
- 故障排除
- 最佳实践
- 计费说明
- 隐私安全

## 技术架构

```
用户请求
    ↓
AI 分析 → 决定调用 web_search
    ↓
Agent Loop → execute_tool("web_search", args)
    ↓
检查 API Key (从数据库)
    ↓
构建 HTTP 请求
    ↓
POST https://iqs.aliyuncs.com/api/v1/search
    ↓
解析响应 JSON
    ↓
格式化结果
    ↓
返回给 AI
    ↓
AI 生成最终回答
    ↓
流式输出给用户
```

## 配置流程

### 用户侧

1. 访问阿里云 IQS 控制台获取 API Key
2. 打开 CoSurf 设置 → 工具
3. 粘贴 API Key 并保存
4. 开始使用 web_search 功能

### 开发侧

无需额外配置，代码已集成完毕。

## API 调用示例

### 请求

```http
POST https://iqs.aliyuncs.com/api/v1/search
Content-Type: application/json
Authorization: Bearer YOUR_API_KEY

{
  "query": "人工智能最新进展",
  "engineType": "News",
  "timeRange": "OneWeek",
  "maxResults": 5
}
```

### 响应

```json
{
  "results": [
    {
      "title": "OpenAI 发布 GPT-5",
      "url": "https://example.com/news1",
      "snippet": "OpenAI 今天发布了新一代语言模型...",
      "rank": 1
    },
    // ... 更多结果
  ]
}
```

## 错误处理

### 1. API Key 未配置

```
错误：未配置阿里云 IQS API Key。
请在设置 → 工具中配置 ALIYUN_IQS_API_KEY。

获取方式：访问 https://help.aliyun.com/zh/document_detail/3025781.html
```

### 2. API 请求失败

```
IQS API 请求失败 (401): Invalid API Key
```

### 3. 无搜索结果

```
未找到相关搜索结果。
```

### 4. 响应格式异常

```
IQS API 返回格式异常，未找到 results 字段。
```

## 性能优化

1. **超时控制**: 30秒超时，避免长时间等待
2. **结果缓存**: 可在未来添加结果缓存机制
3. **并发限制**: 单次调用最多20条结果
4. **日志记录**: 详细日志便于调试和监控

## 安全性

1. **API Key 存储**: 存储在本地 SQLite 数据库
2. **密码输入**: UI 中使用 password 类型隐藏输入
3. **HTTPS 通信**: 所有 API 调用使用 HTTPS
4. **权限控制**: 仅授权用户可配置 API Key

## 测试建议

### 1. 配置测试

```bash
# 1. 启动应用
pnpm dev:full

# 2. 打开设置 → 工具
# 3. 输入测试 API Key
# 4. 点击保存
# 5. 确认显示"API Key 已配置"
```

### 2. 功能测试

```
用户: 帮我搜索今天的科技新闻

预期:
- AI 调用 web_search 工具
- 传入参数: query="科技新闻", engine_type="News", time_range="OneDay"
- 返回格式化的搜索结果
- AI 基于结果生成回答
```

### 3. 错误测试

**测试场景 1**: 未配置 API Key
```
用户: 搜索最新新闻
预期: 返回错误提示，引导用户配置 API Key
```

**测试场景 2**: 无效 API Key
```
用户: 搜索天气
预期: 返回 API 错误信息
```

**测试场景 3**: 无结果
```
用户: 搜索 xyz123abc（不存在的词）
预期: 返回"未找到相关搜索结果"
```

## 已知限制

1. **需要 API Key**: 用户必须自行申请阿里云 IQS API Key
2. **计费**: IQS 服务按调用次数计费
3. **网络依赖**: 需要稳定的网络连接
4. **地域限制**: 可能受地域访问限制

## 未来改进

1. **多搜索引擎支持**: 支持切换不同的搜索引擎
2. **结果缓存**: 缓存常见查询的结果
3. **批量搜索**: 支持同时执行多个搜索
4. **搜索历史**: 记录用户的搜索历史
5. **智能推荐**: 根据上下文自动推荐搜索参数

## 相关文件清单

### 新增文件
- `examples/alibabacloud-iqs-search-skill.json` - IQS Skill 示例
- `docs/IQS_CONFIGURATION.md` - 配置指南

### 修改文件
- `packages/shared/src/settings.ts` - 添加 iqsApiKey 字段
- `src-web/src/components/settings/SettingsPage.tsx` - 添加配置 UI
- `src-tauri/src/ai/tools.rs` - 更新工具 Schema
- `src-tauri/src/ai/stream.rs` - 实现 IQS API 调用
- `src-tauri/src/commands/ai.rs` - 更新系统提示词
- `README.md` - 添加工具说明

## 总结

成功将 Web Search 工具切换到阿里云 IQS，实现了：

✅ 完整的前端配置界面  
✅ 安全的 API Key 管理  
✅ 强大的搜索能力（支持多种引擎和时间范围）  
✅ 完善的错误处理和用户提示  
✅ 详细的文档和示例  

用户现在可以通过简单的配置，享受阿里云 IQS 提供的专业搜索服务，大幅提升 AI 助手的信息获取能力。

---

**下一步**: 测试功能并收集用户反馈，持续优化搜索体验。
