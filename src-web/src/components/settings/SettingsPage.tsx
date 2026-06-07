import {
  X,
  Monitor,
  Cpu,
  Wrench,
  Keyboard,
  Sun,
  Moon,
  MonitorSmartphone,
  Trash2,
  Plus,
  Edit,
  Code,
  Server,
} from "lucide-react";
import { useUIStore, type SettingsView } from "@/stores/uiStore";
import { useSettingsStore } from "@/stores/settingsStore";
import { IconButton } from "@/components/ui/IconButton";
import { cn } from "@/lib/utils";
import { MODEL_PROVIDER_PRESETS, BUILT_IN_TOOLS } from "@cosurf/shared";
import type { ThemeMode, ModelConfig } from "@cosurf/shared";
import { SkillsSettings } from "./SkillsSettings";
import { McpServersSettings } from "./McpServersSettings";
import { useState, useEffect } from "react";

const navItems: { id: SettingsView; icon: typeof Monitor; label: string }[] = [
  { id: "general", icon: Monitor, label: "常规" },
  { id: "models", icon: Cpu, label: "模型" },
  { id: "tools", icon: Wrench, label: "工具" },
  { id: "skills", icon: Code, label: "Skills" },
  { id: "mcp", icon: Server, label: "MCP Servers" },
  { id: "shortcuts", icon: Keyboard, label: "快捷键" },
];

export function SettingsPage() {
  const settingsOpen = useUIStore((s) => s.settingsOpen);
  const closeSettings = useUIStore((s) => s.closeSettings);
  const settingsView = useUIStore((s) => s.settingsView);
  const setSettingsView = useUIStore((s) => s.setSettingsView);
  const loadModels = useSettingsStore((s) => s.loadModels);
  const loadSkillsDirectory = useSettingsStore((s) => s.loadSkillsDirectory);
  const loadIqsApiKey = useSettingsStore((s) => s.loadIqsApiKey);

  // 当设置页面打开时，加载模型列表
  useEffect(() => {
    if (settingsOpen && settingsView === "models") {
      loadModels();
    }
  }, [settingsOpen, settingsView, loadModels]);

  // 当切换到 skills 标签时，加载 Skills 目录配置
  useEffect(() => {
    if (settingsOpen && settingsView === "skills") {
      console.log('[SettingsPage] Switched to skills tab, loading directory...');
      loadSkillsDirectory();
    }
  }, [settingsOpen, settingsView, loadSkillsDirectory]);

  // 当切换到 tools 标签时，加载 IQS API Key
  useEffect(() => {
    if (settingsOpen && settingsView === "tools") {
      console.log('[SettingsPage] Switched to tools tab, loading IQS config...');
      loadIqsApiKey();
    }
  }, [settingsOpen, settingsView, loadIqsApiKey]);

  if (!settingsOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 animate-fade-in">
      <div className="w-[748px] h-[528px] bg-surface rounded-xl shadow-2xl border border-border flex overflow-hidden">
        <div className="w-[160px] bg-surface-secondary border-r border-border flex flex-col">
          <div className="px-4 py-3 border-b border-border">
            <span className="text-sm font-semibold">设置</span>
          </div>
          <nav className="flex-1 py-2">
            {navItems.map((item) => {
              const Icon = item.icon;
              return (
                <button
                  key={item.id}
                  onClick={() => setSettingsView(item.id)}
                  className={cn(
                    "w-full flex items-center gap-2 px-4 py-2 text-xs transition-colors",
                    settingsView === item.id
                      ? "bg-surface-active text-content font-medium"
                      : "text-content-secondary hover:bg-surface-hover hover:text-content",
                  )}
                >
                  <Icon className="w-4 h-4" />
                  {item.label}
                </button>
              );
            })}
          </nav>
        </div>

        <div className="flex-1 flex flex-col">
          <div className="flex items-center justify-between px-4 py-3 border-b border-border">
            <span className="text-sm font-medium">
              {navItems.find((n) => n.id === settingsView)?.label}
            </span>
            <IconButton size="sm" onClick={closeSettings}>
              <X />
            </IconButton>
          </div>
          <div className="flex-1 overflow-y-auto p-4">
            {settingsView === "general" && <GeneralSettings />}
            {settingsView === "models" && <ModelSettings />}
            {settingsView === "tools" && <ToolSettings />}
            {settingsView === "skills" && <SkillsSettings />}
            {settingsView === "mcp" && <McpServersSettings />}
            {settingsView === "shortcuts" && <ShortcutSettings />}
          </div>
        </div>
      </div>
    </div>
  );
}

function GeneralSettings() {
  const settings = useSettingsStore((s) => s.settings);
  const setTheme = useSettingsStore((s) => s.setTheme);
  const setLanguage = useSettingsStore((s) => s.setLanguage);
  const setUserName = useSettingsStore((s) => s.setUserName);
  const updateSettings = useSettingsStore((s) => s.updateSettings);

  const themes: { value: ThemeMode; icon: typeof Sun; label: string }[] = [
    { value: "light", icon: Sun, label: "浅色" },
    { value: "dark", icon: Moon, label: "深色" },
    { value: "system", icon: MonitorSmartphone, label: "跟随系统" },
  ];

  return (
    <div className="space-y-6">
      <SettingGroup label="用户名称">
        <div className="flex items-center gap-2">
          <input
            type="text"
            value={settings.userName || "CoCo"}
            onChange={(e) => setUserName(e.target.value)}
            placeholder="输入用户名称"
            className="flex-1 px-3 py-2 rounded-lg text-xs border border-border bg-surface-secondary text-content outline-none focus:border-brand-500 transition-colors"
            maxLength={20}
          />
          <span className="text-2xs text-content-tertiary">
            将显示在对话中
          </span>
        </div>
      </SettingGroup>

      <SettingGroup label="主题">
        <div className="flex gap-2">
          {themes.map((t) => {
            const Icon = t.icon;
            return (
              <button
                key={t.value}
                onClick={() => setTheme(t.value)}
                className={cn(
                  "flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs border transition-colors",
                  settings.theme === t.value
                    ? "border-brand-500 bg-brand-500/10 text-brand-600"
                    : "border-border hover:border-border-secondary",
                )}
              >
                <Icon className="w-4 h-4" />
                {t.label}
              </button>
            );
          })}
        </div>
      </SettingGroup>

      <SettingGroup label="语言">
        <div className="flex gap-2">
          {([
            { value: "zh-CN" as const, label: "简体中文" },
            { value: "en-US" as const, label: "English" },
          ]).map((lang) => (
            <button
              key={lang.value}
              onClick={() => setLanguage(lang.value)}
              className={cn(
                "px-3 py-2 rounded-lg text-xs border transition-colors",
                settings.language === lang.value
                  ? "border-brand-500 bg-brand-500/10 text-brand-600"
                  : "border-border hover:border-border-secondary",
              )}
            >
              {lang.label}
            </button>
          ))}
        </div>
      </SettingGroup>

      <SettingGroup label="字体大小">
        <input
          type="range"
          min={12}
          max={18}
          value={settings.fontSize}
          onChange={(e) =>
            updateSettings({ fontSize: Number(e.target.value) })
          }
          className="w-48 accent-brand-600"
        />
        <span className="text-xs text-content-secondary ml-2">
          {settings.fontSize}px
        </span>
      </SettingGroup>

      <SettingGroup label="AI 面板默认高度">
        <input
          type="range"
          min={200}
          max={600}
          step={20}
          value={settings.panelDefaultHeight}
          onChange={(e) =>
            updateSettings({ panelDefaultHeight: Number(e.target.value) })
          }
          className="w-48 accent-brand-600"
        />
        <span className="text-xs text-content-secondary ml-2">
          {settings.panelDefaultHeight}px
        </span>
      </SettingGroup>

      <SettingGroup label="隐私模式">
        <ToggleSwitch
          checked={settings.privacyMode}
          onChange={(v) => updateSettings({ privacyMode: v })}
        />
        <span className="text-xs text-content-secondary ml-2">
          启用后不保存浏览历史
        </span>
      </SettingGroup>
    </div>
  );
}

function ModelSettings() {
  const models = useSettingsStore((s) => s.models);
  const activeModelId = useSettingsStore((s) => s.activeModelId);
  const setActiveModel = useSettingsStore((s) => s.setActiveModel);
  const removeModel = useSettingsStore((s) => s.removeModel);
  const [showAdd, setShowAdd] = useState(false);
  const [editingModel, setEditingModel] = useState<ModelConfig | null>(null);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <span className="text-xs text-content-secondary">
          已配置的 AI 模型
        </span>
        <button
          onClick={() => {
            setEditingModel(null);
            setShowAdd(!showAdd);
          }}
          className="flex items-center gap-1 px-2.5 py-1 rounded-md text-xs bg-brand-600 text-white hover:bg-brand-700 transition-colors"
        >
          <Plus className="w-3 h-3" />
          添加模型
        </button>
      </div>

      {showAdd && (
        <AddModelForm
          model={editingModel}
          onDone={() => {
            setShowAdd(false);
            setEditingModel(null);
          }}
        />
      )}

      <div className="space-y-2">
        {models.map((model) => (
          <div
            key={model.id}
            className={cn(
              "flex items-center gap-3 p-3 rounded-lg border transition-colors cursor-pointer",
              model.id === activeModelId
                ? "border-brand-500 bg-brand-500/5"
                : "border-border hover:border-border-secondary",
            )}
            onClick={() => setActiveModel(model.id)}
          >
            <div
              className={cn(
                "w-4 h-4 rounded-full border-2 flex items-center justify-center",
                model.id === activeModelId
                  ? "border-brand-500"
                  : "border-border-secondary",
              )}
            >
              {model.id === activeModelId && (
                <div className="w-2 h-2 rounded-full bg-brand-500" />
              )}
            </div>
            <div className="flex-1 min-w-0">
              <div className="text-xs font-medium">{model.name}</div>
              <div className="text-2xs text-content-tertiary">
                {model.provider} · {model.modelId}
                {model.isLocal ? " · 本地" : ""}
              </div>
            </div>
            <div className="flex items-center gap-1">
              <IconButton
                size="sm"
                onClick={(e) => {
                  e.stopPropagation();
                  setEditingModel(model);
                  setShowAdd(true);
                }}
              >
                <Edit className="w-3 h-3" />
              </IconButton>
              <IconButton
                size="sm"
                onClick={(e) => {
                  e.stopPropagation();
                  removeModel(model.id);
                }}
              >
                <Trash2 className="w-3 h-3" />
              </IconButton>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function AddModelForm({ model, onDone }: { model?: ModelConfig | null; onDone: () => void }) {
  const addModel = useSettingsStore((s) => s.addModel);
  const updateModel = useSettingsStore((s) => s.updateModel);
  const isEditing = !!model;

  const [provider, setProvider] = useState<ModelConfig["provider"]>(
    model?.provider ?? MODEL_PROVIDER_PRESETS[0]!.provider,
  );
  const [name, setName] = useState(model?.name ?? "");
  const [modelId, setModelId] = useState(model?.modelId ?? "");
  const [apiKey, setApiKey] = useState(model?.apiKey ?? "");
  const [baseUrl, setBaseUrl] = useState(
    model?.baseUrl ??
      MODEL_PROVIDER_PRESETS.find((p) => p.provider === provider)?.defaultBaseUrl ??
      "",
  );
  const [temperature, setTemperature] = useState(model?.temperature ?? 0.7);
  const [topP, setTopP] = useState(model?.topP ?? 1);
  const [maxTokens, setMaxTokens] = useState(model?.maxTokens ?? 4096);
  const [isLocal, setIsLocal] = useState(
    model?.isLocal ??
      MODEL_PROVIDER_PRESETS.find((p) => p.provider === provider)?.isLocal ??
      false,
  );

  // 当提供商改变时，更新默认值
  useEffect(() => {
    if (!isEditing) {
      const preset = MODEL_PROVIDER_PRESETS.find((p) => p.provider === provider);
      if (preset) {
        setBaseUrl(preset.defaultBaseUrl);
        setModelId(preset.models[0] ?? "");
        setIsLocal(preset.isLocal);
        if (!name) {
          setName(`${preset.name} - ${preset.models[0]}`);
        }
      }
    }
  }, [provider, isEditing]);

  const handleSubmit = async () => {
    try {
      const modelData = {
        name: name || `${provider}-${modelId}`,
        provider,
        modelId,
        apiKey: apiKey || undefined,
        baseUrl: baseUrl || undefined,
        temperature,
        topP,
        maxTokens,
        isLocal,
        isActive: false,
      };

      if (isEditing && model) {
        await updateModel(model.id, modelData);
      } else {
        await addModel(modelData);
      }
      onDone();
    } catch (error) {
      console.error("Failed to save model:", error);
    }
  };

  return (
    <div className="p-3 rounded-lg border border-border bg-surface-secondary space-y-3">
      <div className="text-xs font-medium mb-2">
        {isEditing ? "编辑模型" : "添加新模型"}
      </div>

      {/* 提供商选择 */}
      {!isEditing && (
        <div>
          <label className="text-2xs font-medium text-content-secondary block mb-1">
            服务提供商
          </label>
          <select
            value={provider}
            onChange={(e) =>
              setProvider(e.target.value as ModelConfig["provider"])
            }
            className="w-full h-8 px-2 rounded-md border border-border bg-surface text-xs outline-none focus:border-brand-500"
          >
            {MODEL_PROVIDER_PRESETS.map((p) => (
              <option key={p.provider} value={p.provider}>
                {p.name}
              </option>
            ))}
          </select>
        </div>
      )}

      {/* 模型名称 */}
      <div>
        <label className="text-2xs font-medium text-content-secondary block mb-1">
          显示名称
        </label>
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="例如：GPT-4"
          className="w-full h-8 px-2 rounded-md border border-border bg-surface text-xs outline-none focus:border-brand-500"
        />
      </div>

      {/* 模型 ID */}
      <div>
        <label className="text-2xs font-medium text-content-secondary block mb-1">
          模型 ID
        </label>
        <input
          type="text"
          value={modelId}
          onChange={(e) => setModelId(e.target.value)}
          placeholder="例如：gpt-4o"
          className="w-full h-8 px-2 rounded-md border border-border bg-surface text-xs outline-none focus:border-brand-500"
        />
      </div>

      {/* API Key */}
      <div>
        <label className="text-2xs font-medium text-content-secondary block mb-1">
          API Key
        </label>
        <input
          type="password"
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
          placeholder="sk-..."
          className="w-full h-8 px-2 rounded-md border border-border bg-surface text-xs outline-none focus:border-brand-500"
        />
      </div>

      {/* Base URL */}
      <div>
        <label className="text-2xs font-medium text-content-secondary block mb-1">
          Base URL
        </label>
        <input
          type="text"
          value={baseUrl}
          onChange={(e) => setBaseUrl(e.target.value)}
          placeholder="https://api.example.com/v1"
          className="w-full h-8 px-2 rounded-md border border-border bg-surface text-xs outline-none focus:border-brand-500"
        />
      </div>

      {/* 参数设置 */}
      <div className="grid grid-cols-3 gap-2">
        <div>
          <label className="text-2xs font-medium text-content-secondary block mb-1">
            Temperature
          </label>
          <input
            type="number"
            min={0}
            max={2}
            step={0.1}
            value={temperature}
            onChange={(e) => setTemperature(Number(e.target.value))}
            className="w-full h-8 px-2 rounded-md border border-border bg-surface text-xs outline-none focus:border-brand-500"
          />
        </div>
        <div>
          <label className="text-2xs font-medium text-content-secondary block mb-1">
            Top P
          </label>
          <input
            type="number"
            min={0}
            max={1}
            step={0.1}
            value={topP}
            onChange={(e) => setTopP(Number(e.target.value))}
            className="w-full h-8 px-2 rounded-md border border-border bg-surface text-xs outline-none focus:border-brand-500"
          />
        </div>
        <div>
          <label className="text-2xs font-medium text-content-secondary block mb-1">
            Max Tokens
          </label>
          <input
            type="number"
            min={1}
            max={128000}
            value={maxTokens}
            onChange={(e) => setMaxTokens(Number(e.target.value))}
            className="w-full h-8 px-2 rounded-md border border-border bg-surface text-xs outline-none focus:border-brand-500"
          />
        </div>
      </div>

      {/* 本地模型开关 */}
      <div className="flex items-center gap-2">
        <input
          type="checkbox"
          id="isLocal"
          checked={isLocal}
          onChange={(e) => setIsLocal(e.target.checked)}
          className="w-4 h-4 accent-brand-600"
        />
        <label htmlFor="isLocal" className="text-2xs text-content-secondary">
          本地模型（如 Ollama）
        </label>
      </div>

      {/* 按钮 */}
      <div className="flex gap-2 pt-2">
        <button
          onClick={handleSubmit}
          className="px-3 py-1.5 rounded-md text-xs bg-brand-600 text-white hover:bg-brand-700"
        >
          {isEditing ? "保存" : "添加"}
        </button>
        <button
          onClick={onDone}
          className="px-3 py-1.5 rounded-md text-xs border border-border hover:bg-surface-hover"
        >
          取消
        </button>
      </div>
    </div>
  );
}

function ToolSettings() {
  const storeIqsApiKey = useSettingsStore((s) => s.iqsApiKey);
  const setIqsApiKey = useSettingsStore((s) => s.setIqsApiKey);
  const [iqsApiKey, setIqsApiKeyLocal] = useState("");
  const [saveStatus, setSaveStatus] = useState<"idle" | "saving" | "success" | "error">("idle");
  const [errorMessage, setErrorMessage] = useState("");

  // 同步 store 中的值到本地状态
  useEffect(() => {
    console.log('[ToolSettings] storeIqsApiKey changed:', storeIqsApiKey ? '***' + storeIqsApiKey.slice(-4) : 'empty');
    setIqsApiKeyLocal(storeIqsApiKey || "");
  }, [storeIqsApiKey]);

  // 保存 IQS API Key
  const saveIqsApiKey = async () => {
    try {
      console.log('[ToolSettings] Saving IQS API Key...');
      setSaveStatus("saving");
      setErrorMessage("");
      await setIqsApiKey(iqsApiKey);
      console.log('[ToolSettings] IQS API Key saved successfully');
      setSaveStatus("success");
      
      // 3秒后清除成功状态
      setTimeout(() => {
        setSaveStatus("idle");
      }, 3000);
    } catch (error) {
      console.error('[ToolSettings] Failed to save IQS API Key:', error);
      setSaveStatus("error");
      setErrorMessage(String(error));
      
      // 5秒后清除错误状态
      setTimeout(() => {
        setSaveStatus("idle");
        setErrorMessage("");
      }, 5000);
    }
  };

  return (
    <div className="space-y-4">
      {/* IQS 配置 */}
      <div>
        <h3 className="text-xs font-medium mb-2">阿里云 IQS 搜索配置</h3>
        <div className="p-3 bg-surface-secondary border border-border rounded-lg space-y-2">
          <div className="text-2xs text-content-secondary">
            配置阿里云智能查询服务(IQS) API Key，用于实时网页搜索。
            <a 
              href="https://help.aliyun.com/zh/document_detail/3025781.html" 
              target="_blank" 
              rel="noopener noreferrer"
              className="text-blue-500 hover:underline ml-1"
            >
              获取 API Key →
            </a>
          </div>
          
          <div className="flex gap-2">
            <input
              type="password"
              value={iqsApiKey}
              onChange={(e) => setIqsApiKeyLocal(e.target.value)}
              placeholder="输入 ALIYUN_IQS_API_KEY"
              className="flex-1 px-2 py-1.5 text-xs bg-surface border border-border rounded-md focus:outline-none focus:border-primary"
            />
            <button
              onClick={saveIqsApiKey}
              disabled={!iqsApiKey.trim() || saveStatus === "saving"}
              className={`px-3 py-1.5 text-xs rounded-md flex items-center gap-1 transition-colors ${
                saveStatus === "success"
                  ? "bg-green-500 text-white"
                  : saveStatus === "error"
                  ? "bg-red-500 text-white"
                  : saveStatus === "saving"
                  ? "bg-primary/50 text-primary-foreground cursor-wait"
                  : "bg-primary text-primary-foreground hover:bg-primary/90"
              } disabled:opacity-50`}
            >
              {saveStatus === "saving" && (
                <span className="animate-spin">⏳</span>
              )}
              {saveStatus === "success" && (
                <span>✓</span>
              )}
              {saveStatus === "error" && (
                <span>✗</span>
              )}
              {saveStatus === "idle" && "保存"}
              {saveStatus === "saving" && "保存中..."}
              {saveStatus === "success" && "已保存"}
              {saveStatus === "error" && "失败"}
            </button>
          </div>
          
          {/* 错误提示 */}
          {errorMessage && (
            <div className="text-2xs text-red-600 dark:text-red-400">
              {errorMessage}
            </div>
          )}
          
          {/* 成功提示 */}
          {storeIqsApiKey && saveStatus !== "error" && (
            <div className="text-2xs text-green-600 dark:text-green-400 flex items-center gap-1">
              <span>✓</span>
              <span>API Key 已配置并保存</span>
            </div>
          )}
        </div>
      </div>

      {/* 内置工具列表 */}
      <div>
        <h3 className="text-xs font-medium mb-2">内置 AI 工具</h3>
        <div className="space-y-2">
          {BUILT_IN_TOOLS.map((tool) => (
            <div
              key={tool.id}
              className="flex items-center gap-3 p-3 rounded-lg border border-border"
            >
              <div className="w-8 h-8 rounded-lg bg-surface-tertiary flex items-center justify-center">
                <Wrench className="w-4 h-4 text-content-secondary" />
              </div>
              <div className="flex-1 min-w-0">
                <div className="text-xs font-medium">{tool.name}</div>
                <div className="text-2xs text-content-tertiary">
                  {tool.description}
                </div>
              </div>
              <ToggleSwitch checked={tool.enabled} onChange={() => {}} />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function ShortcutSettings() {
  const settings = useSettingsStore((s) => s.settings);
  const shortcutLabels: Record<string, string> = {
    togglePanel: "切换 AI 面板",
    newTab: "新建标签页",
    closeTab: "关闭标签页",
    focusAddressBar: "聚焦地址栏",
    newConversation: "新建对话",
    screenshot: "截图",
  };

  return (
    <div className="space-y-3">
      {Object.entries(settings.shortcuts).map(([key, value]) => (
        <div
          key={key}
          className="flex items-center justify-between py-2 border-b border-border last:border-0"
        >
          <span className="text-xs">
            {shortcutLabels[key] ?? key}
          </span>
          <kbd className="px-2 py-0.5 rounded bg-surface-secondary border border-border text-2xs font-mono text-content-secondary">
            {value}
          </kbd>
        </div>
      ))}
    </div>
  );
}

function SettingGroup({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div>
      <label className="text-xs font-medium text-content-secondary block mb-2">
        {label}
      </label>
      <div className="flex items-center">{children}</div>
    </div>
  );
}

function ToggleSwitch({
  checked,
  onChange,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <button
      onClick={() => onChange(!checked)}
      className={cn(
        "relative w-9 h-5 rounded-full transition-colors",
        checked ? "bg-brand-600" : "bg-surface-tertiary",
      )}
    >
      <div
        className={cn(
          "absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-transform",
          checked ? "translate-x-4" : "translate-x-0.5",
        )}
      />
    </button>
  );
}
