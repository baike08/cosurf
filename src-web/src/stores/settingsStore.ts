import { create } from "zustand";
import type { AppSettings, ModelConfig, ThemeMode, Language } from "@cosurf/shared";
import { DEFAULT_SETTINGS } from "@cosurf/shared";
import { invoke } from "@/lib/tauri";

interface BackendModelConfig {
  id: string;
  name: string;
  provider: string;
  modelId: string;
  apiKey?: string;
  baseUrl?: string;
  temperature: number;
  topP: number;
  maxTokens: number;
  isLocal: boolean;
  isActive: boolean;
}

interface SettingsState {
  settings: AppSettings;
  models: ModelConfig[];
  activeModelId: string;
  isLoading: boolean;
  // Skills 配置
  skillsDirectory: string;
  // IQS API Key (独立配置)
  iqsApiKey: string;

  loadModels: () => Promise<void>;
  loadSkillsDirectory: () => Promise<void>;
  loadIqsApiKey: () => Promise<void>;
  setTheme: (theme: ThemeMode) => void;
  setLanguage: (lang: Language) => void;
  setUserName: (name: string) => void;
  updateSettings: (partial: Partial<AppSettings>) => void;
  setActiveModel: (id: string) => Promise<void>;
  addModel: (model: Omit<ModelConfig, "id">) => Promise<void>;
  removeModel: (id: string) => Promise<void>;
  updateModel: (id: string, updates: Partial<ModelConfig>) => Promise<void>;
  // Skills 配置方法
  setSkillsDirectory: (directory: string) => Promise<void>;
  // IQS API Key 方法
  setIqsApiKey: (apiKey: string) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  settings: DEFAULT_SETTINGS,
  models: [],
  activeModelId: "",
  isLoading: false,
  skillsDirectory: "",
  iqsApiKey: "",

  loadModels: async () => {
    try {
      set({ isLoading: true });
      const models = await invoke<BackendModelConfig[]>("list_model_configs");
      const activeModel = await invoke<BackendModelConfig | null>("get_active_model");
      
      set({
        models: models as ModelConfig[],
        activeModelId: activeModel?.id ?? "",
        isLoading: false,
      });
    } catch (error) {
      console.error("Failed to load models:", error);
      set({ isLoading: false });
    }
  },

  setTheme: (theme) => {
    set((state) => ({
      settings: { ...state.settings, theme },
    }));
  },

  setLanguage: (language) => {
    set((state) => ({
      settings: { ...state.settings, language },
    }));
  },

  setUserName: (userName) => {
    set((state) => ({
      settings: { ...state.settings, userName },
    }));
  },

  updateSettings: async (partial) => {
    // 先更新前端状态
    set((state) => ({
      settings: { ...state.settings, ...partial },
    }));
    
    // 然后保存到后端数据库
    try {
      for (const [key, value] of Object.entries(partial)) {
        if (value !== undefined && value !== null) {
          await invoke("set_setting", { 
            key, 
            value: String(value) 
          });
        }
      }
    } catch (error) {
      console.error("Failed to save settings:", error);
    }
  },

  setActiveModel: async (id) => {
    try {
      await invoke("set_active_model", { id });
      set({ activeModelId: id });
    } catch (error) {
      console.error("Failed to set active model:", error);
    }
  },

  addModel: async (modelData) => {
    try {
      const newModel = await invoke<BackendModelConfig>("create_model_config", {
        request: {
          name: modelData.name,
          provider: modelData.provider,
          modelId: modelData.modelId,
          apiKey: modelData.apiKey,
          baseUrl: modelData.baseUrl,
          temperature: modelData.temperature,
          topP: modelData.topP,
          maxTokens: modelData.maxTokens,
          isLocal: modelData.isLocal,
        },
      });
      
      set((state) => ({
        models: [...state.models, newModel as ModelConfig],
      }));
    } catch (error) {
      console.error("Failed to add model:", error);
      throw error;
    }
  },

  removeModel: async (id) => {
    try {
      await invoke("delete_model_config", { id });
      set((state) => ({
        models: state.models.filter((m) => m.id !== id),
        activeModelId:
          state.activeModelId === id
            ? (state.models.find((m) => m.id !== id)?.id ?? "")
            : state.activeModelId,
      }));
    } catch (error) {
      console.error("Failed to remove model:", error);
    }
  },

  updateModel: async (id, updates) => {
    try {
      const updatedModel = await invoke<BackendModelConfig>("update_model_config", {
        id,
        request: {
          name: updates.name,
          apiKey: updates.apiKey,
          baseUrl: updates.baseUrl,
          temperature: updates.temperature,
          topP: updates.topP,
          maxTokens: updates.maxTokens,
        },
      });
      
      set((state) => ({
        models: state.models.map((m) =>
          m.id === id ? (updatedModel as ModelConfig) : m,
        ),
      }));
    } catch (error) {
      console.error("Failed to update model:", error);
      throw error;
    }
  },

  // 加载 Skills 目录配置
  loadSkillsDirectory: async () => {
    try {
      console.log('[loadSkillsDirectory] Loading skills directory...');
      const skillsDir = await invoke<string>("get_skills_directory");
      console.log('[loadSkillsDirectory] Skills directory:', skillsDir);
      
      set({ skillsDirectory: skillsDir });
      console.log('[loadSkillsDirectory] Directory loaded successfully');
    } catch (error) {
      console.error("Failed to load skills directory:", error);
    }
  },

  // 加载 IQS API Key
  loadIqsApiKey: async () => {
    try {
      console.log('[loadIqsApiKey] Loading IQS API Key...');
      const iqsKey = await invoke<string | null>("get_iqs_api_key");
      console.log('[loadIqsApiKey] IQS API Key loaded:', iqsKey ? '***' + iqsKey.slice(-4) : 'null');
      
      set({ iqsApiKey: iqsKey || "" });
      console.log('[loadIqsApiKey] API Key loaded successfully');
    } catch (error) {
      console.error("Failed to load IQS API key:", error);
    }
  },

  // 设置 Skills 目录
  setSkillsDirectory: async (directory) => {
    try {
      await invoke("set_skills_directory", { directory });
      set({ skillsDirectory: directory });
    } catch (error) {
      console.error("Failed to set skills directory:", error);
    }
  },

  // 设置 IQS API Key
  setIqsApiKey: async (apiKey) => {
    try {
      console.log('[setIqsApiKey] Setting IQS API Key, length:', apiKey.length);
      await invoke("set_iqs_api_key", { apiKey });
      console.log('[setIqsApiKey] API Key saved to database');
      set({ iqsApiKey: apiKey });
      console.log('[setIqsApiKey] Store updated');
    } catch (error) {
      console.error("Failed to set IQS API key:", error);
    }
  },
}));
