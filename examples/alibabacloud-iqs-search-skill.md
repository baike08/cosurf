---
id: alibabacloud-iqs-search
name: 阿里云 IQS 智能搜索
description: 使用阿里云智能查询服务(IQS)进行实时网页搜索
type: script
enabled: true
tags:
  - search
  - web
  - aliyun
  - iqs
---

# 阿里云 IQS 智能搜索

这个 Skill 使用阿里云智能查询服务 (Intelligent Query Service) 进行实时网页搜索，提供最新的搜索结果。

## 配置

```yaml
language: python
is_file: false
timeout: 30
source: |
  import sys, json, urllib.request, urllib.error, os

  # 读取参数
  args_file = sys.argv[1] if len(sys.argv) > 1 else None
  params = {}
  if args_file:
      with open(args_file, 'r') as f:
          params = json.load(f)

  query = params.get('query', '')
  if not query:
      print(json.dumps({"error": "Missing query parameter"}))
      sys.exit(1)

  # 从环境变量或系统设置获取 API Key
  api_key = os.environ.get('ALIYUN_IQS_API_KEY', '')
  if not api_key:
      print(json.dumps({"error": "ALIYUN_IQS_API_KEY not set. Configure in Settings > Tools"}))
      sys.exit(1)

  # 构建请求
  request_body = json.dumps({
      "query": query,
      "engineType": "Generic",
      "timeRange": params.get('freshness', 'noLimit'),
      "advancedParam": {"numResults": params.get('numResults', 5)}
  }).encode('utf-8')

  req = urllib.request.Request(
      "https://cloud-iqs.aliyuncs.com/search/unified",
      data=request_body,
      headers={
          "Content-Type": "application/json",
          "Authorization": f"Bearer {api_key}"
      },
      method="POST"
  )

  try:
      with urllib.request.urlopen(req, timeout=30) as resp:
          result = json.loads(resp.read().decode('utf-8'))
          items = result.get("items", [])
          output = []
          for i, item in enumerate(items[:10], 1):
              output.append({
                  "rank": i,
                  "title": item.get("title", ""),
                  "url": item.get("link", ""),
                  "summary": item.get("summary", "")
              })
          print(json.dumps({"results": output, "total": len(items)}, ensure_ascii=False))
  except urllib.error.HTTPError as e:
      print(json.dumps({"error": f"HTTP {e.code}: {e.read().decode('utf-8', errors='replace')}"}))
  except Exception as e:
      print(json.dumps({"error": str(e)}))
```

## 参数

| 参数名 | 类型 | 必填 | 默认值 | 描述 |
|--------|------|------|--------|------|
| query | string | 是 | - | 搜索查询词 |
| numResults | integer | 否 | 5 | 返回结果数量 (1-10) |
| freshness | string | 否 | noLimit | 时间范围: oneDay, oneWeek, oneMonth, noLimit |

## 使用示例

```typescript
// 基本搜索
使用 alibabacloud-iqs-search，query="AI 最新进展"

// 指定结果数量
使用 alibabacloud-iqs-search，query="机器学习教程", numResults=10

// 限制时间范围
使用 alibabacloud-iqs-search，query="科技新闻", freshness="oneDay"
```

## 环境变量

使用前需要设置以下环境变量：

```bash
export ALIYUN_IQS_API_KEY="your-api-key-here"
```

或者在设置 → 工具中配置 IQS API Key。

## 响应格式

成功时返回 JSON 格式的搜索结果：

```json
{
  "results": [
    {
      "rank": 1,
      "title": "文章标题",
      "url": "https://example.com/article",
      "summary": "内容摘要..."
    }
  ],
  "total": 100
}
```

## 错误处理

可能的错误：

- `401 Unauthorized`: API Key 无效或未设置
- `429 Too Many Requests`: 请求频率超限
- `500 Internal Server Error`: 服务器内部错误
- `Timeout`: 请求超时（30秒）

## 注意事项

1. **API Key 安全**: 永远不要将 API Key 硬编码在代码中，使用环境变量
2. **速率限制**: 注意阿里云的 API 调用频率限制
3. **结果缓存**: 相同的查询会被缓存，避免重复调用
4. **超时控制**: 默认超时时间为 30 秒
