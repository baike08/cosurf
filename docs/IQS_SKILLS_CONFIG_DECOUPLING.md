# IQS API Key 与 Skills 配置解耦重构

## 概述

本次重构将 IQS API Key 配置和 Skills 目录配置从耦合状态分离为两个独立的配置项,遵循单一职责原则。

## 问题

### 原设计的问题

```typescript
// ❌ 耦合的加载方法
loadSkillsConfig: async () => {
  const skillsDir = await invoke("get_skills_directory");
  const iqsKey = await invoke("get_iqs_api_key");  // 不应该在这里加载
  
  set({ 
    skillsDirectory: skillsDir,
    iqsApiKey: iqsKey || "" 
  });
}
```

**问题**:
1. **违反单一职责**: 一个方法负责加载两个不相关的配置
2. **不必要的耦合**: IQS API Key 是工具配置,Skills 目录是 Skills 管理配置
3. **性能浪费**: 切换到 Skills 标签时会加载不需要的 IQS API Key
4. **维护困难**: 修改一个配置可能影响另一个

## 解决方案

### 1. Store 层解耦

**文件**: `src-web/src/stores/settingsStore.ts`

```typescript
interface SettingsState {
  // ... 其他字段
  
  // Skills 配置
  skillsDirectory: string;
  
  // IQS API Key (独立配置)
  iqsApiKey: string;

  // 分离的加载方法
  loadSkillsDirectory: () => Promise<void>;
  loadIqsApiKey: () => Promise<void>;
  
  // 设置方法
  setSkillsDirectory: (directory: string) => Promise<void>;
  setIqsApiKey: (apiKey: string) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  // ... 其他状态和方法
  
  // ✅ 只加载 Skills 目录
  loadSkillsDirectory: async () => {
    try {
      const skillsDir = await invoke<string>("get_skills_directory");
      set({ skillsDirectory: skillsDir });
    } catch (error) {
      console.error("Failed to load skills directory:", error);
    }
  },

  // ✅ 只加载 IQS API Key
  loadIqsApiKey: async () => {
    try {
      const iqsKey = await invoke<string | null>("get_iqs_api_key");
      set({ iqsApiKey: iqsKey || "" });
    } catch (error) {
      console.error("Failed to load IQS API key:", error);
    }
  },
  
  // ... 其他方法
}));
```

### 2. 父组件按需加载

**文件**: `src-web/src/components/settings/SettingsPage.tsx`

```tsx
export function SettingsPage() {
  const settingsView = useUIStore((s) => s.settingsView);
  const loadSkillsDirectory = useSettingsStore((s) => s.loadSkillsDirectory);
  const loadIqsApiKey = useSettingsStore((s) => s.loadIqsApiKey);

  // ✅ 只在切换到 Skills 标签时加载目录
  useEffect(() => {
    if (settingsOpen && settingsView === "skills") {
      loadSkillsDirectory();
    }
  }, [settingsOpen, settingsView, loadSkillsDirectory]);

  // ✅ 只在切换到 Tools 标签时加载 IQS API Key
  useEffect(() => {
    if (settingsOpen && settingsView === "tools") {
      loadIqsApiKey();
    }
  }, [settingsOpen, settingsView, loadIqsApiKey]);

  return (
    <div>
      {settingsView === "skills" && <SkillsSettings />}
      {settingsView === "tools" && <ToolSettings />}
    </div>
  );
}
```

### 3. 子组件简化

**ToolSettings 组件**:
```tsx
function ToolSettings() {
  const settings = useSettingsStore((s) => s.settings);
  const setIqsApiKey = useSettingsStore((s) => s.setIqsApiKey);
  const [iqsApiKey, setIqsApiKeyLocal] = useState("");

  // ✅ 只负责同步 store 到本地状态
  useEffect(() => {
    setIqsApiKeyLocal(settings.iqsApiKey || "");
  }, [settings.iqsApiKey]);

  // 保存逻辑
  const saveIqsApiKey = async () => {
    await setIqsApiKey(iqsApiKey);
  };

  return (/* UI */);
}
```

**SkillsSettings 组件**:
```tsx
function SkillsSettings() {
  // 保留内部的 loadSkillsConfig 方法
  // 因为它需要同时加载目录和文件列表
  const loadSkillsConfig = async () => {
    const dir = await invoke("get_skills_directory");
    const files = await invoke("list_skill_files");
    // ...
  };

  useEffect(() => {
    loadSkills();
    loadSkillsConfig();  // 内部实现,合理
  }, []);

  return (/* UI */);
}
```

## 架构对比

### 重构前

```
SettingsPage
  └─ useEffect (settingsView === "tools")
      └─ loadSkillsConfig()  ❌ 耦合
          ├─ get_skills_directory()
          └─ get_iqs_api_key()
          
Store:
  loadSkillsConfig()  ❌ 一个方法做两件事
```

### 重构后

```
SettingsPage
  ├─ useEffect (settingsView === "skills")
  │   └─ loadSkillsDirectory()  ✅ 独立
  │       └─ get_skills_directory()
  │
  └─ useEffect (settingsView === "tools")
      └─ loadIqsApiKey()  ✅ 独立
          └─ get_iqs_api_key()
          
Store:
  loadSkillsDirectory()  ✅ 单一职责
  loadIqsApiKey()        ✅ 单一职责
```

## 优势

### 1. 单一职责原则

每个方法只负责一个配置的加载:
- `loadSkillsDirectory()` → 只加载 Skills 目录
- `loadIqsApiKey()` → 只加载 IQS API Key

### 2. 按需加载

只在需要时加载对应的配置:
- 切换到 "Skills" 标签 → 只加载目录
- 切换到 "Tools" 标签 → 只加载 API Key

### 3. 性能优化

避免不必要的网络请求:
- 重构前: 每次切换到任一标签都发起 2 个请求
- 重构后: 每次切换只发起 1 个请求

### 4. 易于维护

修改一个配置不影响另一个:
- 修改 Skills 目录加载逻辑 → 不影响 IQS API Key
- 修改 IQS API Key 加载逻辑 → 不影响 Skills 目录

### 5. 清晰的职责划分

- **Store**: 提供独立的加载方法
- **Parent Component**: 协调加载时机
- **Child Component**: 只负责展示

## 数据流

### Skills 目录

```
用户切换到 "Skills" 标签
  ↓
SettingsPage useEffect 触发
  ↓
loadSkillsDirectory()
  ↓
后端: get_skills_directory()
  ↓
Store: skillsDirectory 更新
  ↓
SkillsSettings 渲染
  ↓
内部调用 loadSkillsConfig() 加载文件列表
```

### IQS API Key

```
用户切换到 "Tools" 标签
  ↓
SettingsPage useEffect 触发
  ↓
loadIqsApiKey()
  ↓
后端: get_iqs_api_key()
  ↓
Store: iqsApiKey 更新
  ↓
ToolSettings 渲染
  ↓
useEffect 同步到本地状态
  ↓
输入框显示 API Key
```

## 测试验证

### 1. IQS API Key 独立加载

```bash
# 打开控制台
1. 打开设置页面
2. 切换到 "Tools" 标签
3. 查看日志: 应该只看到 [loadIqsApiKey] 相关日志
4. 不应该看到 [loadSkillsDirectory] 日志
```

### 2. Skills 目录独立加载

```bash
# 打开控制台
1. 打开设置页面
2. 切换到 "Skills" 标签
3. 查看日志: 应该只看到 [loadSkillsDirectory] 相关日志
4. 不应该看到 [loadIqsApiKey] 日志
```

### 3. 配置持久化

```bash
1. 配置 IQS API Key 并保存
2. 关闭应用
3. 重新启动
4. 打开设置 → "Tools" 标签
5. ✅ 应该看到之前配置的 API Key
```

## 相关文件

- ✅ `src-web/src/stores/settingsStore.ts` - Store 层解耦
- ✅ `src-web/src/components/settings/SettingsPage.tsx` - 父组件协调
- ✅ `src-web/src/components/settings/SkillsSettings.tsx` - Skills 组件(保持内部实现)
- ✅ `docs/IQS_API_KEY_PERSISTENCE_FIX.md` - 详细修复文档

## 注意事项

1. **SkillsSettings 保留内部方法**: `SkillsSettings` 组件有自己的 `loadSkillsConfig()` 方法,这是合理的,因为它需要同时加载目录和文件列表。

2. **不要重复加载**: 避免在父子组件中都调用同一个加载函数。

3. **依赖数组要完整**: 确保 `useEffect` 包含所有依赖。

4. **错误处理**: 每个加载方法都有独立的错误处理。

## 未来扩展

如果需要添加新的配置项,遵循相同模式:

```typescript
// 1. Store 中添加
loadNewConfig: async () => {
  const value = await invoke("get_new_config");
  set({ newConfig: value });
},

// 2. Parent Component 中添加
useEffect(() => {
  if (settingsOpen && settingsView === "new-tab") {
    loadNewConfig();
  }
}, [settingsOpen, settingsView, loadNewConfig]);

// 3. Child Component 中展示
useEffect(() => {
  setLocalState(store.newConfig);
}, [store.newConfig]);
```

## 总结

通过本次重构:
- ✅ 解耦了 IQS API Key 和 Skills 目录配置
- ✅ 遵循单一职责原则
- ✅ 实现了按需加载
- ✅ 提升了性能和可维护性
- ✅ 保持了代码的清晰性和可扩展性
