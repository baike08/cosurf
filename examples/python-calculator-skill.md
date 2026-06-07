---
id: python-calculator
name: Python 计算器
description: 使用 Python 执行数学计算
type: script
enabled: true
tags:
  - calculator
  - math
  - python
---

# Python 计算器

这个技能使用 Python 执行各种数学计算，支持加减乘除、幂运算、三角函数等。

## 配置

```yaml
language: python
source: |
  import sys
  import json
  import math
  
  # 读取参数
  args_file = sys.argv[1]
  with open(args_file, 'r') as f:
      params = json.load(f)
  
  expression = params.get('expression', '')
  
  try:
      # 安全地评估表达式
      result = eval(expression, {"__builtins__": {}}, {
          'math': math,
          'sin': math.sin,
          'cos': math.cos,
          'tan': math.tan,
          'sqrt': math.sqrt,
          'pi': math.pi,
          'e': math.e
      })
      print(json.dumps({
          'success': True,
          'result': result,
          'expression': expression
      }))
  except Exception as e:
      print(json.dumps({
          'success': False,
          'error': str(e),
          'expression': expression
      }))
is_file: false
timeout: 10
```

## 参数

| 参数名 | 类型 | 必填 | 默认值 | 描述 |
|--------|------|------|--------|------|
| expression | string | 是 | - | 数学表达式，如 "2 + 2" 或 "math.sin(math.pi/2)" |

## 使用示例

```python
# 基本计算
使用 python-calculator，expression="2 + 3 * 4"
# 输出: {"success": true, "result": 14, "expression": "2 + 3 * 4"}

# 三角函数
使用 python-calculator，expression="math.sin(math.pi/2)"
# 输出: {"success": true, "result": 1.0, "expression": "math.sin(math.pi/2)"}

# 平方根
使用 python-calculator，expression="math.sqrt(16)"
# 输出: {"success": true, "result": 4.0, "expression": "math.sqrt(16)"}
```

## 安全说明

- 该技能禁用了 Python 的内置函数，只允许访问 `math` 模块
- 不支持文件操作、网络请求等危险操作
- 超时时间设置为 10 秒，防止无限循环

## 支持的函数

- 基本运算：`+`, `-`, `*`, `/`, `**` (幂), `%` (取模)
- 三角函数：`sin()`, `cos()`, `tan()`
- 其他函数：`sqrt()` (平方根)
- 常量：`pi`, `e`
