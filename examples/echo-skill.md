---
id: echo-skill
name: Echo 消息
description: 简单的回显技能，用于测试 Skills 系统
type: cli
enabled: true
tags:
  - test
  - demo
---

# Echo 消息

这是一个简单的回显技能，用于演示 CoSurf Skills 系统的基本功能。

## 配置

```yaml
command: echo
args_template:
  - "Hello from {{message}}!"
timeout: 5
require_confirmation: false
```

## 参数

| 参数名 | 类型 | 必填 | 默认值 | 描述 |
|--------|------|------|--------|------|
| message | string | 否 | CoSurf | 要回显的消息 |

## 使用示例

```bash
# 基本用法
使用 echo-skill，message="World"

# 输出
Hello from World!
```

## 注意事项

- 这是一个测试技能，仅用于验证 Skills 系统是否正常工作
- 支持参数模板替换，使用 `{{param_name}}` 语法
