import { useState, useEffect } from "react";
import { Save, RotateCcw, CheckCircle2, XCircle } from "lucide-react";
import { db } from "@/lib/api";

interface AgentPrompt {
  id: string;
  name: string;
  content: string;
  description?: string;
  isEnabled: boolean;
  createdAt: string;
  updatedAt: string;
}

export function AgentPromptsSettings() {
  const [prompts, setPrompts] = useState<AgentPrompt[]>([]);
  const [selectedPrompt, setSelectedPrompt] = useState<string | null>(null);
  const [editingContent, setEditingContent] = useState("");
  const [editingDescription, setEditingDescription] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [saveStatus, setSaveStatus] = useState<"idle" | "success" | "error">("idle");

  // 加载所有 prompts
  useEffect(() => {
    loadPrompts();
  }, []);

  const loadPrompts = async () => {
    try {
      const list = await db.listAgentPrompts();
      setPrompts(list || []);
      if (list && list.length > 0 && !selectedPrompt) {
        setSelectedPrompt(list[0].name);
        setEditingContent(list[0].content);
        setEditingDescription(list[0].description || "");
      }
    } catch (err) {
      console.error("[AgentPrompts] Failed to load prompts:", err);
    }
  };

  // 选择 prompt
  const handleSelect = (prompt: AgentPrompt) => {
    setSelectedPrompt(prompt.name);
    setEditingContent(prompt.content);
    setEditingDescription(prompt.description || "");
    setSaveStatus("idle");
  };

  // 保存修改
  const handleSave = async () => {
    if (!selectedPrompt) return;
    
    setIsSaving(true);
    setSaveStatus("idle");
    
    try {
      await db.setAgentPrompt(selectedPrompt, editingContent, editingDescription);
      setSaveStatus("success");
      
      // 重新加载列表
      setTimeout(() => {
        loadPrompts();
        setSaveStatus("idle");
      }, 1500);
    } catch (err) {
      console.error("[AgentPrompts] Failed to save:", err);
      setSaveStatus("error");
    } finally {
      setIsSaving(false);
    }
  };

  // 重置为原始内容
  const handleReset = () => {
    const prompt = prompts.find(p => p.name === selectedPrompt);
    if (prompt) {
      setEditingContent(prompt.content);
      setEditingDescription(prompt.description || "");
      setSaveStatus("idle");
    }
  };

  // 切换启用状态
  const handleToggle = async (name: string) => {
    try {
      await db.toggleAgentPrompt(name);
      loadPrompts();
    } catch (err) {
      console.error("[AgentPrompts] Failed to toggle:", err);
    }
  };

  const currentPrompt = prompts.find(p => p.name === selectedPrompt);

  return (
    <div className="h-full flex gap-4">
      {/* 左侧：Prompts 列表 */}
      <div className="w-64 border-r border-border pr-4 overflow-y-auto">
        <h3 className="text-sm font-semibold mb-3 text-content">Agent Prompts</h3>
        <div className="space-y-2">
          {prompts.map((prompt) => (
            <button
              key={prompt.id}
              onClick={() => handleSelect(prompt)}
              className={`w-full text-left px-3 py-2 rounded-lg transition-colors ${
                selectedPrompt === prompt.name
                  ? "bg-surface-active text-content font-medium"
                  : "hover:bg-surface-hover text-content-secondary"
              }`}
            >
              <div className="flex items-center justify-between">
                <span className="text-xs truncate">{prompt.name}</span>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleToggle(prompt.name);
                  }}
                  className="ml-2"
                  title={prompt.isEnabled ? "禁用" : "启用"}
                >
                  {prompt.isEnabled ? (
                    <CheckCircle2 className="w-3 h-3 text-green-500" />
                  ) : (
                    <XCircle className="w-3 h-3 text-gray-500" />
                  )}
                </button>
              </div>
              {prompt.description && (
                <p className="text-[10px] text-content-tertiary mt-1 truncate">
                  {prompt.description}
                </p>
              )}
            </button>
          ))}
        </div>
      </div>

      {/* 右侧：编辑器 */}
      <div className="flex-1 flex flex-col">
        {currentPrompt ? (
          <>
            <div className="mb-4">
              <h4 className="text-sm font-medium text-content mb-1">
                {currentPrompt.name}
              </h4>
              <p className="text-xs text-content-tertiary">
                {currentPrompt.description || "无描述"}
              </p>
            </div>

            <div className="flex-1 flex flex-col gap-3">
              <div>
                <label className="text-xs font-medium text-content-secondary mb-1 block">
                  描述
                </label>
                <input
                  type="text"
                  value={editingDescription}
                  onChange={(e) => setEditingDescription(e.target.value)}
                  placeholder="简短描述这个 Prompt 的用途..."
                  className="w-full px-3 py-2 bg-surface-secondary border border-border rounded-lg text-xs text-content placeholder:text-content-tertiary focus:outline-none focus:ring-2 focus:ring-primary/50"
                />
              </div>

              <div className="flex-1 flex flex-col">
                <label className="text-xs font-medium text-content-secondary mb-1 block">
                  Prompt 内容
                </label>
                <textarea
                  value={editingContent}
                  onChange={(e) => setEditingContent(e.target.value)}
                  className="flex-1 w-full px-3 py-2 bg-surface-secondary border border-border rounded-lg text-xs text-content font-mono resize-none focus:outline-none focus:ring-2 focus:ring-primary/50"
                  placeholder="输入 System Prompt 内容..."
                />
              </div>

              <div className="flex items-center justify-between pt-2">
                <div className="flex items-center gap-2">
                  {saveStatus === "success" && (
                    <span className="text-xs text-green-500 flex items-center gap-1">
                      <CheckCircle2 className="w-3 h-3" />
                      已保存
                    </span>
                  )}
                  {saveStatus === "error" && (
                    <span className="text-xs text-red-500 flex items-center gap-1">
                      <XCircle className="w-3 h-3" />
                      保存失败
                    </span>
                  )}
                </div>

                <div className="flex gap-2">
                  <button
                    onClick={handleReset}
                    disabled={isSaving}
                    className="px-3 py-1.5 text-xs text-content-secondary hover:text-content hover:bg-surface-hover rounded-lg transition-colors disabled:opacity-50"
                  >
                    <RotateCcw className="w-3 h-3 inline mr-1" />
                    重置
                  </button>
                  <button
                    onClick={handleSave}
                    disabled={isSaving || !editingContent.trim()}
                    className="px-3 py-1.5 text-xs bg-primary text-white hover:bg-primary/90 rounded-lg transition-colors disabled:opacity-50 flex items-center gap-1"
                  >
                    <Save className="w-3 h-3" />
                    {isSaving ? "保存中..." : "保存"}
                  </button>
                </div>
              </div>
            </div>
          </>
        ) : (
          <div className="flex-1 flex items-center justify-center text-content-tertiary text-sm">
            选择一个 Prompt 开始编辑
          </div>
        )}
      </div>
    </div>
  );
}
