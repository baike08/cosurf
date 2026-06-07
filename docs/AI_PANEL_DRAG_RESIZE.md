# AIPanel 拖拽调整宽度功能实现

## 🎯 功能概述

实现了 AIPanel（AI 对话面板）的拖拽调整宽度功能，用户可以通过拖拽面板左侧边缘来调整面板宽度。

## ✅ 实现内容

### 1. 状态管理 (uiStore)

**文件**: `src-web/src/stores/uiStore.ts`

#### 修改内容

**重命名字段**:
- `aiPanelHeight` → `aiPanelWidth` （更准确的命名）
- `setAIPanelHeight` → `setAIPanelWidth`

**新增方法**:
```typescript
setAIPanelWidth: (width) => {
  // 最小300px，最大窗口宽度的60%
  const maxWidth = Math.floor(window.innerWidth * 0.6);
  const clampedWidth = Math.max(300, Math.min(width, maxWidth));
  set({ aiPanelWidth: clampedWidth });
}
```

**默认值**:
- `aiPanelWidth: 400` - 默认宽度 400px

### 2. UI 组件 (AIPanel)

**文件**: `src-web/src/components/layout/AIPanel.tsx`

#### 拖拽手柄实现

```tsx
{/* 拖拽手柄 */}
<div
  className="w-1 bg-transparent hover:bg-brand-500/20 cursor-col-resize shrink-0 transition-colors"
  onMouseDown={(e) => {
    e.preventDefault();
    const startX = e.clientX;
    const startWidth = aiPanelWidth;

    const handleMouseMove = (moveEvent: MouseEvent) => {
      const delta = startX - moveEvent.clientX; // 向左拖动增加宽度
      setAIPanelWidth(startWidth + delta);
    };

    const handleMouseUp = () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
  }}
/>
```

#### 布局结构

```tsx
<div className="flex h-full">
  {/* AI 面板主体 */}
  <div style={{ width: aiPanelWidth }}>
    {/* 面板内容 */}
  </div>

  {/* 拖拽手柄 */}
  <div className="w-1 ...">
    {/* 拖拽逻辑 */}
  </div>
</div>
```

## 🔧 技术细节

### 拖拽逻辑

1. **鼠标按下 (onMouseDown)**
   - 记录起始位置 `startX`
   - 记录起始宽度 `startWidth`
   - 添加全局鼠标移动和释放监听器
   - 设置光标为 `col-resize`
   - 禁止文本选择

2. **鼠标移动 (handleMouseMove)**
   - 计算位移：`delta = startX - moveEvent.clientX`
   - 向左拖动时 delta 为正，增加宽度
   - 向右拖动时 delta 为负，减少宽度
   - 调用 `setAIPanelWidth` 更新宽度

3. **鼠标释放 (handleMouseUp)**
   - 移除全局事件监听器
   - 恢复光标样式
   - 恢复文本选择

### 宽度限制

- **最小宽度**: 300px - 确保面板内容可读
- **最大宽度**: 窗口宽度的 60% - 确保浏览器区域可见

### 视觉反馈

- **正常状态**: 透明背景 (`bg-transparent`)
- **悬停状态**: 品牌色高亮 (`hover:bg-brand-500/20`)
- **拖拽状态**: 光标变为 `col-resize`
- **过渡效果**: `transition-colors` 平滑过渡

## 📁 修改的文件

1. **`src-web/src/stores/uiStore.ts`**
   - 重命名 `aiPanelHeight` → `aiPanelWidth`
   - 重命名 `setAIPanelHeight` → `setAIPanelWidth`
   - 更新宽度限制逻辑（300px - 60%）

2. **`src-web/src/components/layout/AIPanel.tsx`**
   - 更新状态引用
   - 添加拖拽手柄
   - 实现拖拽逻辑
   - 调整布局结构

## 🎨 用户体验

### 交互流程

1. 用户将鼠标移动到 AIPanel 左侧边缘
2. 边缘高亮显示（品牌色半透明）
3. 光标变为左右箭头（col-resize）
4. 用户按住鼠标左键并拖动
5. 面板宽度实时跟随鼠标移动
6. 释放鼠标完成调整

### 视觉提示

- **悬停高亮**: 提示用户可以拖拽
- **光标变化**: 明确指示拖拽方向
- **实时反馈**: 宽度变化即时可见
- **边界限制**: 防止过度调整

## 🔍 与 Sidebar 的对比

| 特性 | Sidebar | AIPanel |
|------|---------|---------|
| 位置 | 左侧 | 右侧 |
| 默认宽度 | 280px | 400px |
| 最小宽度 | 200px | 300px |
| 最大宽度 | 窗口 50% | 窗口 60% |
| 拖拽方向 | 向右增加 | 向左增加 |
| 实现方式 | 相同 | 相同 |

## 💡 使用示例

### 编程方式调整宽度

```typescript
import { useUIStore } from "@/stores/uiStore"

function MyComponent() {
  const setAIPanelWidth = useUIStore((s) => s.setAIPanelWidth)
  
  const makePanelWider = () => {
    setAIPanelWidth(600) // 设置为 600px
  }
  
  return <button onClick={makePanelWider}>扩大面板</button>
}
```

### 获取当前宽度

```typescript
const aiPanelWidth = useUIStore((s) => s.aiPanelWidth)
console.log("当前宽度:", aiPanelWidth)
```

## ⚠️ 注意事项

### 1. 跨窗口尺寸

当窗口大小改变时，最大宽度限制会动态调整：
```typescript
const maxWidth = Math.floor(window.innerWidth * 0.6)
```

**建议**: 可以添加窗口 resize 监听器，自动调整超出限制的宽度。

### 2. 性能优化

当前实现在每次鼠标移动时都会触发状态更新和重新渲染。

**优化方案**（可选）:
```typescript
// 使用 requestAnimationFrame 节流
let rafId: number
const handleMouseMove = (moveEvent: MouseEvent) => {
  if (rafId) cancelAnimationFrame(rafId)
  rafId = requestAnimationFrame(() => {
    const delta = startX - moveEvent.clientX
    setAIPanelWidth(startWidth + delta)
  })
}
```

### 3. 触摸设备支持

当前实现仅支持鼠标事件。如需支持触摸设备：

```typescript
// 添加触摸事件支持
onTouchStart={(e) => {
  const touch = e.touches[0]
  // 类似鼠标逻辑
}}
```

## 🎯 下一步优化

### 高优先级
- [ ] 添加窗口 resize 监听，自动调整超限宽度
- [ ] 保存宽度到 localStorage，重启后保持
- [ ] 添加双击重置功能

### 中优先级
- [ ] 支持触摸设备
- [ ] 添加动画过渡效果
- [ ] 显示当前宽度提示

### 低优先级
- [ ] 支持键盘快捷键调整
- [ ] 预设宽度选项（小/中/大）
- [ ] 记忆多个会话的不同宽度

## 📊 测试清单

- [x] 拖拽手柄正确显示
- [x] 悬停时高亮效果
- [x] 拖拽时光标变化
- [x] 宽度实时更新
- [x] 最小宽度限制生效
- [x] 最大宽度限制生效
- [x] 释放鼠标后停止调整
- [x] 不影响其他交互

## 🔗 相关文档

- [Sidebar 拖拽调整宽度](./sidebar-drag-resize.md)
- [UI Store 状态管理](../src-web/src/stores/uiStore.ts)
- [AIPanel 组件](../src-web/src/components/layout/AIPanel.tsx)

---

**实现日期**: 2026-05-23  
**版本**: v1.0  
**状态**: ✅ 已完成
