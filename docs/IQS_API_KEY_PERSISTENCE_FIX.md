# IQS API Key 配置持久化问题修复

## 问题描述

用户在设置页面配置 IQS API Key 后:
1. ✅ 保存成功(显示成功提示)
2. ❌ 关闭设置页面后重新打开
3. ❌ 切换到"工具"标签页,看不到之前配置的 API Key

## 根本原因

### 问题分析

**1. 组件生命周期问题**:

```tsx
// SettingsPage.tsx - 条件渲染
{settingsView === "tools" && <ToolSettings />}
```

当用户:
1. 在"工具"标签页保存 API Key
2. 切换到其他标签(如"模型") → `ToolSettings` **被卸载**
3. 关闭设置页面 → 整个 `SettingsPage` **被卸载**
4. 重新打开设置页面 → `SettingsPage` **重新挂载**
5. 默认显示"常规"标签 → `ToolSettings` **未挂载**
6. 切换到"工具"标签 → `ToolSettings` **重新挂载**

**2. 配置耦合问题** (已修复):

原代码中 `loadSkillsConfig()` 同时加载了:
- Skills 目录配置
- IQS API Key

这导致两个独立的功能耦合在一起,违反了单一职责原则。

**原代码的问题**:

```typescript
// settingsStore.ts - 耦合的加载方法
loadSkillsConfig: async () => {
  const skillsDir = await invoke("get_skills_directory");
  const iqsKey = await invoke("get_iqs_api_key");  // ❌ 不应该在这里加载
  
  set({ skillsDirectory: skillsDir, iqsApiKey: iqsKey });
}
```

## 解决方案

### 方案 1: 在父组件级别监听标签切换

在 `SettingsPage` 组件中添加对 `settingsView` 变化的监听:

```tsx
export function SettingsPage() {
  const settingsView = useUIStore((s) => s.settingsView);
  const loadSkillsDirectory = useSettingsStore((s) => s.loadSkillsDirectory);
  const loadIqsApiKey = useSettingsStore((s) => s.loadIqsApiKey);

  // 当切换到 skills 标签时，加载 Skills 目录配置
  useEffect(() => {
    if (settingsOpen && settingsView === "skills") {
      loadSkillsDirectory();
    }
  }, [settingsOpen, settingsView, loadSkillsDirectory]);

  // 当切换到 tools 标签时，加载 IQS API Key
  useEffect(() => {
    if (settingsOpen && settingsView === "tools") {
      loadIqsApiKey();
    }
  }, [settingsOpen, settingsView, loadIqsApiKey]);

  return (
    // ...
    {settingsView === "skills" && <SkillsSettings />}
    {settingsView === "tools" && <ToolSettings />}
  );
}
```

### 方案 2: 解耦配置加载 (已实施)

将 `loadSkillsConfig()` 拆分为两个独立的方法:

```typescript
// settingsStore.ts - 分离的加载方法

// 只加载 Skills 目录
loadSkillsDirectory: async () => {
  const skillsDir = await invoke("get_skills_directory");
  set({ skillsDirectory: skillsDir });
},

// 只加载 IQS API Key
loadIqsApiKey: async () => {
  const iqsKey = await invoke("get_iqs_api_key");
  set({ iqsApiKey: iqsKey || "" });
},
```

### 优势

1. **单一职责**: 每个方法只负责一个配置的加载
2. **明确的触发时机**: 每次切换到对应标签都会重新加载配置
3. **不依赖子组件生命周期**: 由父组件统一管理数据加载
4. **与 models 标签保持一致**: 采用相同的模式
5. **更可靠的同步**: 确保每次进入标签页都获取最新数据
6. **易于维护**: 修改一个配置不影响另一个

### 优化子组件

移除 `ToolSettings` 中的重复加载逻辑:

```tsx
function ToolSettings() {
  const settings = useSettingsStore((s) => s.settings);
  const setIqsApiKey = useSettingsStore((s) => s.setIqsApiKey);
  const [iqsApiKey, setIqsApiKeyLocal] = useState("");

  // 只负责同步 store 到本地状态
  useEffect(() => {
    setIqsApiKeyLocal(settings.iqsApiKey || "");
  }, [settings.iqsApiKey]);

  // 保存逻辑保持不变
  const saveIqsApiKey = async () => {
    await setIqsApiKey(iqsApiKey);
  };

  return (/* UI */);
}
```

**注意**: `SkillsSettings` 组件保留了自己的 `loadSkillsConfig()` 方法,
因为它需要同时加载目录和文件列表,这是合理的内部实现。

## 修改文件

### 1. `src-web/src/stores/settingsStore.ts`

**重构**:
- ❌ 移除: `loadSkillsConfig()` (耦合的方法)
- ✅ 新增: `loadSkillsDirectory()` (只加载 Skills 目录)
- ✅ 新增: `loadIqsApiKey()` (只加载 IQS API Key)

**原因**: 解耦两个独立的配置,遵循单一职责原则

### 2. `src-web/src/components/settings/SettingsPage.tsx`

**添加**:
- 导入 `loadSkillsDirectory` 和 `loadIqsApiKey`
- 新增 `useEffect` 监听 `settingsView === "skills"`
- 新增 `useEffect` 监听 `settingsView === "tools"`

**移除**:
- `ToolSettings` 中的 `loadSkillsConfig` 导入
- `ToolSettings` 中的加载 `useEffect`

### 3. 相关数据流

**Skills 目录加载流程**:
```
用户切换到"Skills"标签
  ↓
SettingsPage useEffect 触发
  ↓
调用 loadSkillsDirectory()
  ↓
从后端获取: get_skills_directory()
  ↓
更新 Store: skillsDirectory
  ↓
SkillsSettings 组件渲染
  ↓
内部调用 loadSkillsConfig() 加载文件列表
```

**IQS API Key 加载流程**:
```
用户切换到"工具"标签
  ↓
SettingsPage useEffect 触发
  ↓
调用 loadIqsApiKey()
  ↓
从后端获取: get_iqs_api_key()
  ↓
更新 Store: iqsApiKey
  ↓
ToolSettings 组件渲染
  ↓
useEffect 同步 store 到本地状态
  ↓
输入框显示 API Key
```

## 测试验证

### 测试场景

#### 1. IQS API Key 配置
- [ ] 打开设置页面
- [ ] 切换到"工具"标签
- [ ] 输入 IQS API Key
- [ ] 点击保存(显示成功)
- [ ] 关闭设置页面
- [ ] 重新打开设置页面
- [ ] 切换到"工具"标签
- [ ] ✅ 应该看到之前配置的 API Key

#### 2. Skills 目录配置
- [ ] 打开设置页面
- [ ] 切换到"Skills"标签
- [ ] 查看当前目录路径
- [ ] 点击编辑按钮
- [ ] 修改目录路径
- [ ] 点击保存
- [ ] 关闭设置页面
- [ ] 重新打开设置页面
- [ ] 切换到"Skills"标签
- [ ] ✅ 应该看到修改后的目录路径

#### 3. 标签切换
- [ ] 在"工具"标签配置 IQS API Key 并保存
- [ ] 切换到"模型"标签
- [ ] 切换到"Skills"标签
- [ ] 切换回"工具"标签
- [ ] ✅ IQS API Key 应该仍然显示
- [ ] ✅ Skills 目录也应该正确显示

#### 4. 多次重启
- [ ] 配置 IQS API Key 并保存
- [ ] 完全关闭应用
- [ ] 重新启动应用
- [ ] 打开设置 → "工具"标签
- [ ] ✅ 应该看到配置的 API Key

### 调试日志

查看控制台输出:

**切换到"工具"标签时**:
```
[SettingsPage] Switched to tools tab, loading IQS config...
[loadIqsApiKey] Loading IQS API Key...
[loadIqsApiKey] IQS API Key loaded: ***xxxx
[loadIqsApiKey] API Key loaded successfully
[ToolSettings] settings.iqsApiKey changed: ***xxxx
```

**切换到"Skills"标签时**:
```
[SettingsPage] Switched to skills tab, loading directory...
[loadSkillsDirectory] Loading skills directory...
[loadSkillsDirectory] Skills directory: /path/to/skills
[loadSkillsDirectory] Directory loaded successfully
```

## 最佳实践

### 配置管理原则

1. **单一职责**: 每个配置项应该有独立的加载方法
2. **按需加载**: 只在需要时加载对应的配置
3. **父组件协调**: 由父组件统一管理加载时机
4. **子组件展示**: 子组件只负责同步和展示

### 示例模式

```typescript
// Store - 分离的加载方法
loadConfigA: async () => {
  const value = await invoke("get_config_a");
  set({ configA: value });
},

loadConfigB: async () => {
  const value = await invoke("get_config_b");
  set({ configB: value });
},

// Parent Component - 按需加载
useEffect(() => {
  if (settingsView === "tab-a") {
    loadConfigA();
  }
}, [settingsView]);

useEffect(() => {
  if (settingsView === "tab-b") {
    loadConfigB();
  }
}, [settingsView]);

// Child Component - 只负责展示
useEffect(() => {
  setLocalState(store.configValue);
}, [store.configValue]);
```

## 类似问题排查

如果其他配置也出现类似问题,检查:

1. **是否在父组件监听了标签切换?**
   ```tsx
   useEffect(() => {
     if (settingsView === "xxx") {
       loadXxxConfig();
     }
   }, [settingsView]);
   ```

2. **Store 是否正确更新?**
   ```tsx
   set({ xxxConfig: newValue });
   ```

3. **子组件是否同步了 store 状态?**
   ```tsx
   useEffect(() => {
     setLocalState(store.xxxConfig);
   }, [store.xxxConfig]);
   ```

## 最佳实践

### 配置加载策略

1. **父组件负责加载**: 在标签切换时触发
2. **子组件负责展示**: 只同步 store 到本地状态
3. **统一管理模式**: 所有标签页采用相同模式

### 示例模式

```tsx
// Parent Component
useEffect(() => {
  if (settingsView === "tab-name") {
    loadTabConfig();
  }
}, [settingsView]);

// Child Component
useEffect(() => {
  setLocalState(store.configValue);
}, [store.configValue]);
```

## 相关文件

- `src-web/src/components/settings/SettingsPage.tsx` - 主设置页面
- `src-web/src/stores/settingsStore.ts` - 状态管理
- `src-tauri/src/commands/settings.rs` - 后端命令
- `src-tauri/src/db/settings.rs` - 数据库操作

## 注意事项

1. **不要重复加载**: 避免在父子组件中都调用加载函数
2. **依赖数组要完整**: 确保 `useEffect` 包含所有依赖
3. **异步加载要考虑竞态**: 使用 loading 状态防止重复请求
4. **错误处理**: 加载失败时要给用户反馈
