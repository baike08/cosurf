# Settings 页面问题修复指南

## 📋 问题概述

修复了 Settings 页面的三个关键问题：

1. ❌ **IQS API Key 重启后丢失** - 页面上看不到已配置的值
2. ❌ **Skills 目录不可见** - 无法看到配置路径和已内置的 Skills
3. ❌ **Skills 导入格式错误** - 应该只支持 Markdown 格式，而非 JSON

---

## ✅ 修复方案

### 问题 1: IQS API Key 持久化显示

#### 原因分析

```typescript
// ❌ 之前的实现
const [iqsApiKey, setIqsApiKey] = useState(settings.iqsApiKey || "");

// 问题：
// 1. 只在组件初始化时读取一次
// 2. loadSkillsConfig() 没有在页面加载时调用
// 3. settings.iqsApiKey 更新后，本地状态不同步
```

#### 修复方案

```typescript
// ✅ 修复后的实现
const loadSkillsConfig = useSettingsStore((s) => s.loadSkillsConfig);
const [iqsApiKey, setIqsApiKey] = useState("");

// 1. 页面加载时调用 loadSkillsConfig
useEffect(() => {
  loadSkillsConfig();
}, []);

// 2. 同步 store 中的值到本地状态
useEffect(() => {
  setIqsApiKey(settings.iqsApiKey || "");
}, [settings.iqsApiKey]);
```

**关键点**：
- ✅ 在 `useEffect` 中调用 `loadSkillsConfig()`
- ✅ 使用另一个 `useEffect` 监听 `settings.iqsApiKey` 变化
- ✅ 确保本地状态与 store 同步

---

### 问题 2: Skills 目录和文件列表显示

#### 原因分析

```typescript
// ❌ 之前只有 Skills 列表，没有目录信息
<div>
  <h3>已安装的 Skills</h3>
  {/* 只显示加载的 Skills */}
</div>

// 问题：
// 1. 看不到配置的目录路径
// 2. 看不到磁盘上的 Skill 文件
// 3. 无法区分哪些文件是 .md，哪些是 .json
```

#### 修复方案

添加两个新区域：

##### 1. Skills 目录显示

```tsx
<div>
  <h3 className="text-sm font-medium mb-2">Skills 目录</h3>
  <div className="p-3 bg-surface-secondary border border-border rounded-lg">
    <div className="flex items-center gap-2 text-xs text-content-secondary mb-2">
      <FolderOpen className="w-4 h-4" />
      <span className="font-mono">{skillsDirectory || "加载中..."}</span>
    </div>
    <div className="text-2xs text-content-secondary">
      所有 Skill Markdown 文件都保存在此目录中
    </div>
  </div>
</div>
```

##### 2. Skill 文件列表

```tsx
<div>
  <h3 className="text-sm font-medium mb-2">Skill 文件 ({skillFiles.length})</h3>
  {skillFiles.length === 0 ? (
    <div className="text-center py-4 ...">
      <FileText className="w-6 h-6 mx-auto mb-1 opacity-50" />
      <p>还没有 Skill 文件</p>
    </div>
  ) : (
    <div className="space-y-1 max-h-48 overflow-y-auto">
      {skillFiles.map((file) => (
        <div key={file.path} className="...">
          <div className="flex items-center gap-2 flex-1">
            <FileText className={`... ${file.isMarkdown ? 'text-blue-500' : 'text-gray-400'}`} />
            <span className="font-mono truncate">{file.filename}</span>
            <span>({(file.size / 1024).toFixed(1)} KB)</span>
          </div>
          <div className="flex items-center gap-2">
            {file.modified && (
              <span>{new Date(file.modified * 1000).toLocaleDateString()}</span>
            )}
            <span className={`... ${file.isMarkdown ? 'MD' : 'JSON'}`}>
          </div>
        </div>
      ))}
    </div>
  )}
</div>
```

**新增功能**：
- ✅ 显示配置的目录路径
- ✅ 列出所有 `.md` 和 `.json` 文件
- ✅ 显示文件大小和修改日期
- ✅ 用颜色区分 Markdown（蓝色）和 JSON（灰色）
- ✅ 滚动容器，最多显示 48px 高度

---

### 问题 3: Skills 导入仅支持 Markdown

#### 原因分析

```typescript
// ❌ 之前支持 JSON 导入
const importFromJson = async () => {
  await invoke("import_skill", { request: { skill_json: skillJson } });
};

const importFromFile = async () => {
  const selected = await open({
    filters: [{ name: "Skill Files", extensions: ["json"] }]
  });
  await invoke("import_skill_from_file", { filePath: selected });
};

// 问题：
// 1. 不符合 Skills Markdown 规范
// 2. JSON 不易阅读和编辑
// 3. 应该统一使用 Markdown 格式
```

#### 修复方案

完全移除 JSON 导入，改为 Markdown：

##### 1. 从文本导入

```typescript
// ✅ 新的实现
const importFromMarkdown = async () => {
  try {
    setLoading(true);
    await invoke("import_skill_from_markdown", { markdownContent });
    setShowImportModal(false);
    setMarkdownContent("");
    await Promise.all([loadSkills(), loadSkillsConfig()]);
    setError(null);
  } catch (err) {
    setError(`Failed to import skill: ${err}`);
  } finally {
    setLoading(false);
  }
};
```

##### 2. 从文件导入

```typescript
// ✅ 新的实现
const importFromMarkdownFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: "Markdown Files",
        extensions: ["md"]  // ← 只接受 .md 文件
      }]
    });
    
    if (selected) {
      setLoading(true);
      await invoke("import_skill_from_markdown_file", { filePath: selected });
      await Promise.all([loadSkills(), loadSkillsConfig()]);
      setError(null);
    }
  } catch (err) {
    setError(`Failed to import from file: ${err}`);
  } finally {
    setLoading(false);
  }
};
```

##### 3. 更新模态框 UI

```tsx
{/* 导入模态框 */}
{showImportModal && (
  <div className="fixed inset-0 z-50 ...">
    <div className="w-[600px] ...">
      <div className="...">
        <h3 className="text-sm font-medium">导入 Skill (Markdown)</h3>
        {/* ... */}
      </div>
      
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        <div>
          <label className="block text-xs font-medium mb-2">
            Skill Markdown 内容
          </label>
          <textarea
            value={markdownContent}
            onChange={(e) => setMarkdownContent(e.target.value)}
            placeholder={`---
id: my-skill
name: My Skill
description: Description here
type: cli
enabled: true
---

# My Skill

## 配置

\`\`\`yaml
command: echo
args_template:
  - "{{message}}"
timeout: 5
\`\`\`

## 参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| message | string | 否 | 消息 |`}
            className="..."
          />
        </div>
        
        <div className="text-xs text-content-secondary">
          <p className="font-medium mb-1">Markdown 格式说明：</p>
          <ul className="list-disc list-inside space-y-1 text-2xs">
            <li>使用 YAML frontmatter 定义元数据</li>
            <li>在代码块中编写 YAML 配置</li>
            <li>使用表格定义参数</li>
            <li>支持 CLI、Script、MCP 等类型</li>
          </ul>
        </div>
      </div>
      
      <div className="...">
        <button onClick={() => setShowImportModal(false)}>取消</button>
        <button
          onClick={importFromMarkdown}
          disabled={!markdownContent.trim()}
        >
          导入
        </button>
      </div>
    </div>
  </div>
)}
```

**改进点**：
- ✅ 标题明确标注 "(Markdown)"
- ✅ Placeholder 显示完整的 Markdown 示例
- ✅ 添加格式说明列表
- ✅ 只接受 `.md` 文件扩展名

---

## 📊 修改的文件

### 前端

1. ✅ [SettingsPage.tsx](file://d:\coding-harness\CoSurf\src-web\src\components\settings\SettingsPage.tsx)
   - 添加 `useEffect` 调用 `loadSkillsConfig()`
   - 添加 `useEffect` 同步 `settings.iqsApiKey` 到本地状态

2. ✅ [SkillsSettings.tsx](file://d:\coding-harness\CoSurf\src-web\src\components\settings\SkillsSettings.tsx)
   - 添加 `SkillFile` 接口
   - 添加 `skillsDirectory` 和 `skillFiles` 状态
   - 添加 `loadSkillsConfig()` 方法
   - 重构导入逻辑为 Markdown 格式
   - 添加 Skills 目录显示区域
   - 添加 Skill 文件列表显示
   - 更新导入模态框为 Markdown 格式

---

## 🎯 用户体验改进

### 之前

```
Settings Page
├─ Tools Tab
│  └─ IQS API Key
│     ├─ 输入框（空）❌
│     └─ 保存按钮
│
└─ Skills Tab
   └─ 已安装的 Skills
      ├─ 导入 Skill (JSON) ❌
      └─ 从文件导入 (.json) ❌
```

**问题**：
- ❌ IQS API Key 重启后为空
- ❌ 看不到 Skills 目录
- ❌ 看不到磁盘上的文件
- ❌ 使用 JSON 格式（不友好）

---

### 现在

```
Settings Page
├─ Tools Tab
│  └─ IQS API Key
│     ├─ 输入框（自动加载已配置的值）✅
│     └─ 保存按钮
│
└─ Skills Tab
   ├─ Skills 目录 ✅
   │  └─ ~/.cosurf/skills/
   │
   ├─ Skill 文件 (5) ✅
   │  ├─ echo-skill.md (1.2 KB, 2026-05-23) [MD]
   │  ├─ calculator.md (2.5 KB, 2026-05-22) [MD]
   │  └─ ...
   │
   └─ 已加载的 Skills (3)
      ├─ 导入 Skill (Markdown) ✅
      └─ 从文件导入 (.md) ✅
```

**改进**：
- ✅ IQS API Key 自动加载并显示
- ✅ 清晰显示配置的目录路径
- ✅ 列出所有 Skill 文件（MD + JSON）
- ✅ 文件大小和修改日期一目了然
- ✅ 使用 Markdown 格式（易读易编辑）

---

## 🧪 测试验证

### 测试 1: IQS API Key 持久化

```typescript
// 1. 设置 API Key
await invoke('set_iqs_api_key', { apiKey: 'sk-test123' });

// 2. 刷新页面或重启应用
// 3. 打开 Settings → Tools
// 4. 验证输入框显示 "sk-test123" ✅
```

---

### 测试 2: Skills 目录显示

```typescript
// 1. 打开 Settings → Skills
// 2. 验证显示：
//    Skills 目录
//    📁 ~/.cosurf/skills/
//    所有 Skill Markdown 文件都保存在此目录中 ✅
```

---

### 测试 3: Skill 文件列表

```typescript
// 1. 在目录中添加测试文件
echo "---
id: test
name: Test
---
# Test" > ~/.cosurf/skills/test.md

// 2. 打开 Settings → Skills
// 3. 验证显示：
//    Skill 文件 (1)
//    📄 test.md (0.1 KB)  2026-05-23  [MD] ✅
```

---

### 测试 4: Markdown 导入

```typescript
// 1. 点击"导入 Skill"
// 2. 粘贴 Markdown 内容：
const markdown = `
---
id: hello
name: Hello World
description: Greeting skill
type: cli
enabled: true
---

# Hello World

## 配置

\`\`\`yaml
command: echo
args_template:
  - "Hello {{name}}!"
timeout: 5
\`\`\`

## 参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| name | string | 否 | 名称 |
`;

// 3. 点击"导入"
// 4. 验证：
//    - Skill 出现在列表中 ✅
//    - 文件保存到 ~/.cosurf/skills/hello.md ✅
//    - 同时生成 hello.json ✅
```

---

### 测试 5: 从文件导入

```typescript
// 1. 创建测试文件
cat > /tmp/my-skill.md << EOF
---
id: my-skill
name: My Skill
---
# My Skill
EOF

// 2. 点击"从文件导入"
// 3. 选择 /tmp/my-skill.md
// 4. 验证：
//    - 文件复制到 ~/.cosurf/skills/my-skill.md ✅
//    - Skill 出现在列表中 ✅
```

---

## 💡 技术要点

### 1. React Hooks 正确使用

```typescript
// ✅ 正确：分离关注点
useEffect(() => {
  loadSkillsConfig();  // 只在挂载时调用
}, []);

useEffect(() => {
  setIqsApiKey(settings.iqsApiKey || "");  // 监听 store 变化
}, [settings.iqsApiKey]);

// ❌ 错误：合并到一个 useEffect
useEffect(() => {
  loadSkillsConfig();
  setIqsApiKey(settings.iqsApiKey || "");
}, []);  // settings.iqsApiKey 变化时不会更新
```

---

### 2. 异步操作并行执行

```typescript
// ✅ 并行加载
await Promise.all([loadSkills(), loadSkillsConfig()]);

// ❌ 串行加载（慢）
await loadSkills();
await loadSkillsConfig();
```

---

### 3. 状态管理最佳实践

```typescript
// ✅ Store 作为单一事实来源
const settings = useSettingsStore((s) => s.settings);
const loadSkillsConfig = useSettingsStore((s) => s.loadSkillsConfig);

// ✅ 本地状态用于 UI 控制
const [iqsApiKey, setIqsApiKey] = useState("");

// ✅ 同步 store 到本地状态
useEffect(() => {
  setIqsApiKey(settings.iqsApiKey || "");
}, [settings.iqsApiKey]);
```

---

## 🚀 后续优化建议

### 1. 目录配置界面

```tsx
<div>
  <h3>Skills 目录</h3>
  <div className="flex gap-2">
    <input value={skillsDirectory} readOnly />
    <button onClick={changeDirectory}>更改</button>
  </div>
</div>
```

---

### 2. 文件操作

```tsx
<button onClick={() => openInExplorer(file.path)}>
  在文件夹中显示
</button>

<button onClick={() => deleteFile(file.path)}>
  删除文件
</button>
```

---

### 3. 实时预览

```tsx
<div className="grid grid-cols-2 gap-4">
  <textarea value={markdownContent} />
  <MarkdownPreview content={markdownContent} />
</div>
```

---

### 4. 批量导入

```tsx
<button onClick={importMultipleFiles}>
  批量导入
</button>

// 支持多选
const selected = await open({
  multiple: true,
  filters: [{ name: "Markdown Files", extensions: ["md"] }]
});
```

---

## 📝 总结

### 修复的问题

1. ✅ **IQS API Key 持久化** - 重启后自动加载并显示
2. ✅ **Skills 目录可见** - 清晰显示配置路径
3. ✅ **文件列表展示** - 列出所有 .md 和 .json 文件
4. ✅ **Markdown 导入** - 统一使用 Markdown 格式

### 关键改进

- 🔄 正确的 React Hooks 使用
- 📂 完整的文件管理视图
- 📝 用户友好的 Markdown 编辑器
- ⚡ 并行加载优化性能

### 用户体验

- 👁️ 可视化更强 - 目录、文件一目了然
- ✍️ 更易编辑 - Markdown 比 JSON 友好
- 💾 更可靠 - 配置持久化无丢失
- 🎯 更直观 - 清晰的标签和提示

---

**最后更新**: 2026-05-23  
**版本**: 1.2.0  
**作者**: CoSurf Team
