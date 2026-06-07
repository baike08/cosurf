---
id: weather-search
name: 天气查询
description: 通过 MCP Server 查询实时天气信息
type: mcp
enabled: true
tags:
  - weather
  - search
  - realtime
---

# 天气查询

这个 Skill 使用 MCP 协议调用天气服务，获取指定城市的实时天气信息。

## 配置

```yaml
server_url: https://weather-mcp-server.example.com
tool_name: get_weather
api_key: ${WEATHER_API_KEY}
```

## 参数

| 参数名 | 类型 | 必填 | 默认值 | 描述 |
|--------|------|------|--------|------|
| city | string | 是 | - | 城市名称（如：北京、Shanghai） |
| units | string | 否 | metric | 单位: metric(摄氏度) 或 imperial(华氏度) |
| lang | string | 否 | zh-CN | 语言: zh-CN, en-US, ja-JP |

## 使用示例

```typescript
// 基本查询
使用 weather-search，city="北京"

// 指定单位
使用 weather-search，city="New York", units="imperial"

// 指定语言
使用 weather-search，city="东京", lang="ja-JP"
```

## 环境变量

```bash
export WEATHER_API_KEY="your-weather-api-key"
```

## 响应格式

```json
{
  "location": {
    "city": "北京",
    "country": "中国",
    "coordinates": {
      "lat": 39.9042,
      "lon": 116.4074
    }
  },
  "current": {
    "temperature": 25,
    "humidity": 60,
    "condition": "晴",
    "wind_speed": 10,
    "wind_direction": "东北"
  },
  "forecast": [
    {
      "date": "2024-01-01",
      "high": 28,
      "low": 18,
      "condition": "多云"
    }
  ]
}
```

## 错误处理

可能的错误：

- `401 Unauthorized`: API Key 无效
- `404 Not Found`: 城市不存在
- `429 Too Many Requests`: 请求频率超限
- `500 Internal Server Error`: 服务器错误
- `Timeout`: 请求超时（30秒）

## 注意事项

1. **API Key 安全**: 使用环境变量存储 API Key
2. **速率限制**: 免费套餐通常有每日调用限制
3. **缓存策略**: 相同城市的查询结果会被缓存 5 分钟
4. **语言支持**: 确保 MCP Server 支持指定的语言
