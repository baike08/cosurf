# CoSurf 浏览器自动化引擎使用指南

## 📚 概述

CoSurf 提供了完整的浏览器自动化能力,让你可以像 Playwright/Puppeteer 一样操作网页。

## 🎯 核心功能

### 1. 元素选择与交互

#### 可视化选择模式
```typescript
// 点击"选择元素"按钮进入选择模式
// 鼠标悬停时元素会高亮显示
// 点击元素后自动捕获 CSS 选择器
```

#### 点击元素
```typescript
// 在 BrowserActionPanel 中:
// 1. 选择一个元素
// 2. 点击"点击元素"按钮
// 3. 引擎会自动执行点击操作
```

#### 输入文本
```typescript
// 在 BrowserActionPanel 中:
// 1. 选择一个输入框
// 2. 在文本框中输入内容
// 3. 点击"输入文本"按钮
```

### 2. 表单操作

#### 自动填充表单
```javascript
// 通过控制台执行
const formData = {
  username: 'your_username',
  password: 'your_password',
  email: 'test@example.com'
};

// 调用自动填充函数
autoFillForm(formData);
```

#### 提交表单
```javascript
// 提交当前页面的表单
submitForm();

// 或提交特定表单
submitForm('#login-form');
```

#### 获取表单字段
```javascript
// 获取当前页面所有表单字段
getFormFields();

// 返回结果示例:
{
  success: true,
  message: "Found 5 form fields",
  data: {
    form: "https://example.com/login",
    fields: [
      {
        name: "username",
        type: "text",
        placeholder: "Enter username",
        required: true,
        selector: "input[name='username']"
      }
    ]
  }
}
```

### 3. 页面导航与控制

#### 滚动页面
```javascript
// 向下滚动 300px
scrollPage('down');

// 向上滚动
scrollPage('up');

// 滚动到顶部
scrollPage('top');

// 滚动到底部
scrollPage('bottom');

// 自定义滚动距离
scrollPage('down', 500);
```

#### 等待元素
```javascript
// 等待元素出现(最多等待 5 秒)
await waitForElement('.loading-spinner', 5000);

// 等待后可以继续操作
const element = await waitForElement('#submit-button');
if (element) {
  clickElement('#submit-button');
}
```

### 4. 内容提取

#### 提取纯文本
```javascript
// 提取整个页面的文本内容
extractPageContent({ format: 'text' });

// 提取特定元素的内容
extractPageContent({ 
  selector: '.article-content',
  format: 'text' 
});
```

#### 提取 HTML
```javascript
// 提取 HTML 结构
extractPageContent({ 
  selector: '#main-content',
  format: 'html' 
});
```

#### 转换为 Markdown
```javascript
// 将页面内容转换为 Markdown 格式
extractPageContent({ format: 'markdown' });

// 返回 Markdown 格式的文本
```

### 5. 高级操作

#### 执行自定义 JavaScript
```javascript
// 执行任意 JavaScript 代码
executeScript(`
  return document.title;
`);

// 修改页面样式
executeScript(`
  document.body.style.backgroundColor = 'red';
`);

// 获取页面信息
executeScript(`
  return {
    url: window.location.href,
    title: document.title,
    cookies: document.cookie
  };
`);
```

#### 高亮元素
```javascript
// 高亮显示某个元素(用于调试)
highlightElement('.important-element', 3000);

// 高亮持续 3 秒后自动消失
```

#### 智能选择器生成
```javascript
// 为元素生成唯一的 CSS 选择器
const selector = generateSmartSelector(element);
// 返回类似: "div.container > button.submit-btn"
```

## 🔧 实际应用场景

### 场景 1: 自动登录网站

```javascript
// 1. 导航到登录页面
// 在地址栏输入: example.com/login

// 2. 等待页面加载完成后,在控制台执行:
autoFillForm({
  username: 'your_username',
  password: 'your_password'
});

// 3. 点击登录按钮
clickElement('#login-button');

// 或者一步到位:
autoFillForm({
  username: 'your_username',
  password: 'your_password'
});
submitForm();
```

### 场景 2: 批量提取数据

```javascript
// 提取所有文章标题和链接
const articles = executeScript(`
  const items = [];
  document.querySelectorAll('.article-item').forEach(el => {
    const title = el.querySelector('.title').textContent;
    const link = el.querySelector('a').href;
    items.push({ title, link });
  });
  return items;
`);

console.log(articles.data.result);
```

### 场景 3: 自动化测试

```javascript
// 测试流程:
// 1. 访问页面
// 2. 填写表单
autoFillForm({
  search: 'test query'
});

// 3. 点击搜索按钮
clickElement('#search-btn');

// 4. 等待结果
await waitForElement('.search-results', 5000);

// 5. 验证结果
const results = extractPageContent({
  selector: '.search-results',
  format: 'text'
});

console.log(results);
```

### 场景 4: 网页截图

```javascript
// 截取当前页面
handleScreenshot();

// TODO: 完整截图功能正在开发中
```

## 📖 API 参考

### 核心函数

| 函数 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `clickElement(selector, options)` | selector: string, options?: object | ActionResult | 点击元素 |
| `inputText(selector, text, options)` | selector: string, text: string, options?: object | ActionResult | 输入文本 |
| `selectOption(selector, value)` | selector: string, value: string \| string[] | ActionResult | 选择下拉选项 |
| `scrollPage(direction, amount?)` | direction: string, amount?: number | ActionResult | 滚动页面 |
| `waitForElement(selector, timeout?)` | selector: string, timeout?: number | Promise<HTMLElement \| null> | 等待元素 |
| `extractPageContent(options)` | options?: object | ActionResult | 提取内容 |
| `getFormFields(formSelector?)` | formSelector?: string | ActionResult | 获取表单字段 |
| `autoFillForm(formData, formSelector?)` | formData: object, formSelector?: string | ActionResult | 自动填充表单 |
| `submitForm(formSelector?)` | formSelector?: string | ActionResult | 提交表单 |
| `executeScript(script)` | script: string | ActionResult | 执行脚本 |
| `highlightElement(selector, duration?)` | selector: string, duration?: number | ActionResult | 高亮元素 |
| `generateSmartSelector(element)` | element: HTMLElement | string | 生成选择器 |

### ActionResult 类型

```typescript
interface ActionResult {
  success: boolean;   // 操作是否成功
  message: string;    // 描述信息
  data?: any;         // 返回数据(可选)
}
```

## 💡 最佳实践

### 1. 使用智能选择器
```javascript
// ❌ 不推荐:脆弱的选择器
clickElement('div > div:nth-child(3) > button');

// ✅ 推荐:使用 ID 或语义化选择器
clickElement('#submit-button');
clickElement('button[type="submit"]');
```

### 2. 添加等待机制
```javascript
// ❌ 不推荐:直接操作可能失败的元素
clickElement('.dynamic-content');

// ✅ 推荐:先等待元素出现
await waitForElement('.dynamic-content', 5000);
clickElement('.dynamic-content');
```

### 3. 错误处理
```javascript
const result = clickElement('#my-button');
if (!result.success) {
  console.error('Click failed:', result.message);
  // 尝试其他方法...
}
```

### 4. 链式操作
```javascript
// 流畅的操作链
await waitForElement('#username');
inputText('#username', 'admin');
inputText('#password', 'secret');
clickElement('#login-btn');
await waitForElement('.dashboard');
```

## 🚀 未来计划

- [ ] 操作录制和回放
- [ ] AI 智能理解用户意图
- [ ] 条件判断和循环
- [ ] 更强大的截图功能
- [ ] PDF 导出
- [ ] 网络请求拦截
- [ ] Cookie 管理
- [ ] 多标签页协同操作

## 📝 注意事项

1. **跨域限制**: iframe 中的跨域页面可能无法访问 DOM
2. **安全策略**: 某些网站有 CSP 保护,可能阻止脚本执行
3. **异步操作**: 使用 `await` 等待异步操作完成
4. **性能考虑**: 大量 DOM 操作可能影响页面性能

## 🆘 常见问题

**Q: 为什么点击按钮没有反应?**
A: 检查选择器是否正确,元素是否在 iframe 中,是否有事件监听器。

**Q: 如何调试选择器?**
A: 使用 `highlightElement(selector)` 高亮显示目标元素。

**Q: 表单提交失败怎么办?**
A: 先调用 `getFormFields()` 检查字段,确保必填字段都已填写。

**Q: 如何提取动态加载的内容?**
A: 使用 `waitForElement()` 等待内容加载完成后再提取。

---

更多问题请参考源代码或联系开发团队。
