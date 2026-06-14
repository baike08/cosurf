import { create } from "zustand";
import type { AppSettings, ModelConfig, ThemeMode, Language } from "@cosurf/shared";
import { DEFAULT_SETTINGS } from "@cosurf/shared";
import { db } from "@/lib/api";

interface SettingsState {
  settings: AppSettings;
  models: ModelConfig[];
  activeModelId: string;
  isLoading: boolean;
  // Skills 配置
  skillsDirectory: string;

  loadModels: () => Promise<void>;
  loadSkillsDirectory: () => Promise<void>;
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
}

export const useSettingsStore = create<SettingsState>((set) => ({
  settings: DEFAULT_SETTINGS,
  models: [],
  activeModelId: "",
  isLoading: false,
  skillsDirectory: "",

  loadModels: async () => {
    try {
      set({ isLoading: true });
      const models = await db.listModelConfigs();
      const activeModel = await db.getActiveModel();
      
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
    set((state) => ({
      settings: { ...state.settings, ...partial },
    }));
    
    try {
      for (const [key, value] of Object.entries(partial)) {
        if (value !== undefined && value !== null) {
          await db.setSetting(key, String(value));
        }
      }
    } catch (error) {
      console.error("Failed to save settings:", error);
    }
  },

  setActiveModel: async (id) => {
    try {
      await db.setActiveModel(id);
      set({ activeModelId: id });
    } catch (error) {
      console.error("Failed to set active model:", error);
    }
  },

  addModel: async (modelData) => {
    try {
      const newModel = await db.createModelConfig({
        name: modelData.name,
        provider: modelData.provider,
        modelId: modelData.modelId,
        apiKey: modelData.apiKey,
        baseUrl: modelData.baseUrl,
        temperature: modelData.temperature,
        topP: modelData.topP,
        maxTokens: modelData.maxTokens,
        isLocal: modelData.isLocal,
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
      await db.deleteModelConfig(id);
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
      const updatedModel = await db.updateModelConfig(id, {
        name: updates.name,
        apiKey: updates.apiKey,
        baseUrl: updates.baseUrl,
        temperature: updates.temperature,
        topP: updates.topP,
        maxTokens: updates.maxTokens,
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

  loadSkillsDirectory: async () => {
    try {
      console.log('[SettingsStore] Loading skills directory...');
      const skillsDir = await db.getSkillsDirectory();
      console.log('[SettingsStore] Skills directory loaded:', skillsDir);
      set({ skillsDirectory: skillsDir || "" });
    } catch (error) {
      console.error("Failed to load skills directory:", error);
    }
  },

  setSkillsDirectory: async (directory) => {
    try {
      await db.setSkillsDirectory(directory);
      set({ skillsDirectory: directory });
    } catch (error) {
      console.error("Failed to set skills directory:", error);
    }
  },
}));
