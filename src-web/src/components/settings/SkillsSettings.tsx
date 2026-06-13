import { useState, useEffect } from "react";
import { X, Plus, Trash2, ToggleLeft, ToggleRight, Code, FolderOpen, FileText, Edit2, Save, Eye } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface Skill {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
  tags: string[];
  dir_path: string;
}

interface SkillDir {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
  tags: string[];
  dir_path: string;
  file_size: number;
  modified?: number;
}

export function SkillsSettings() {
  const [skills, setSkills] = useState<Skill[]>([]);
  const [skillDirs, setSkillDirs] = useState<SkillDir[]>([]);
  const [skillsDirectory, setSkillsDirectory] = useState("");
  const [editingDirectory, setEditingDirectory] = useState(false);
  const [tempDirectory, setTempDirectory] = useState("");
  const [loading, setLoading] = useState(false);
  const [showImportModal, setShowImportModal] = useState(false);
  const [markdownContent, setMarkdownContent] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [previewSkillId, setPreviewSkillId] = useState<string | null>(null);
  const [previewContent, setPreviewContent] = useState<string>("");
  const [loadingPreview, setLoadingPreview] = useState(false);

  // 加载 Skills
  const loadSkills = async () => {
    try {
      setLoading(true);
      const loaded = await invoke<Skill[]>("list_skills");
      setSkills(loaded);
      setError(null);
    } catch (err) {
      setError(`Failed to load skills: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // 加载 Skills 目录信息
  const loadSkillDirs = async () => {
    try {
      const dirs = await invoke<SkillDir[]>("list_skill_files");
      setSkillDirs(dirs);
    } catch (err) {
      console.error("Failed to load skill directories:", err);
    }
  };

  // 加载 Skills 配置
  const loadSkillsConfig = async () => {
    try {
      const dir = await invoke<string>("get_skills_directory");
      setSkillsDirectory(dir);
      setTempDirectory(dir);

      await loadSkillDirs();
    } catch (err) {
      console.error("Failed to load skills config:", err);
    }
  };

  // 选择目录
  const selectDirectory = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "选择 Skills 目录"
      });

      if (selected) {
        setTempDirectory(selected as string);
      }
    } catch (err) {
      setError(`Failed to select directory: ${err}`);
    }
  };

  // 保存目录配置
  const saveDirectory = async () => {
    if (!tempDirectory.trim()) {
      setError("目录路径不能为空");
      return;
    }

    try {
      setLoading(true);
      setError(null);
      await invoke("set_skills_directory", { directory: tempDirectory });
      setSkillsDirectory(tempDirectory);
      setEditingDirectory(false);

      // 重新加载 Skills
      await Promise.all([loadSkills(), loadSkillsConfig()]);
    } catch (err) {
      setError(`Failed to save directory: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // 取消编辑
  const cancelEdit = () => {
    setTempDirectory(skillsDirectory);
    setEditingDirectory(false);
    setError(null);
  };

  // 从文件夹导入 Skill
  const importFromDirectory = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "选择 Skill 文件夹（需包含 SKILL.md）"
      });

      if (selected) {
        setLoading(true);
        setError(null);
        await invoke("import_skill_from_directory", { sourceDir: selected });
        await Promise.all([loadSkills(), loadSkillsConfig()]);
      }
    } catch (err) {
      setError(`Failed to import from directory: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // 从 Markdown 导入
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

  // 预览 Skill 内容
  const previewSkill = async (skillId: string) => {
    try {
      setLoadingPreview(true);
      setPreviewSkillId(skillId);
      const content = await invoke<string>("get_skill_content", { skillId });
      setPreviewContent(content);
    } catch (err) {
      setPreviewContent(`Error loading skill content: ${err}`);
    } finally {
      setLoadingPreview(false);
    }
  };

  // 关闭预览
  const closePreview = () => {
    setPreviewSkillId(null);
    setPreviewContent("");
  };

  // 删除 Skill
  const deleteSkill = async (id: string) => {
    if (!confirm("确定要删除这个 Skill 吗？")) return;

    try {
      setLoading(true);
      await invoke("delete_skill", { skillId: id });
      await Promise.all([loadSkills(), loadSkillDirs()]);
      setError(null);
    } catch (err) {
      setError(`Failed to delete skill: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // 启用/禁用 Skill
  const toggleSkill = async (id: string, enabled: boolean) => {
    try {
      setLoading(true);
      await invoke("toggle_skill", { request: { skill_id: id, enabled } });
      await loadSkills();
      setError(null);
    } catch (err) {
      setError(`Failed to toggle skill: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // 首次加载
  useEffect(() => {
    loadSkills();
    loadSkillsConfig();
  }, []);

  return (
    <div className="space-y-4">
      {/* Skills 目录配置 */}
      <div>
        <h3 className="text-sm font-medium mb-2">Skills 目录</h3>
        {editingDirectory ? (
          <div className="p-3 bg-surface-secondary border border-border rounded-lg space-y-2">
            <div className="flex gap-2">
              <input
                type="text"
                value={tempDirectory}
                onChange={(e) => setTempDirectory(e.target.value)}
                placeholder="输入 Skills 目录路径"
                className="flex-1 px-2 py-1.5 text-xs font-mono bg-surface border border-border rounded-md focus:outline-none focus:border-primary"
              />
              <button
                onClick={selectDirectory}
                className="px-2 py-1.5 text-xs bg-surface-active border border-border rounded-md hover:bg-surface-hover flex items-center gap-1"
                title="选择目录"
              >
                <FolderOpen className="w-3 h-3" />
              </button>
            </div>
            <div className="flex justify-end gap-2">
              <button
                onClick={cancelEdit}
                className="px-3 py-1.5 text-xs border border-border rounded-md hover:bg-surface-hover flex items-center gap-1"
              >
                <X className="w-3 h-3" />
                取消
              </button>
              <button
                onClick={saveDirectory}
                disabled={loading || !tempDirectory.trim()}
                className="px-3 py-1.5 text-xs bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50 flex items-center gap-1"
              >
                <Save className="w-3 h-3" />
                {loading ? "保存中..." : "保存"}
              </button>
            </div>
          </div>
        ) : (
          <div className="p-3 bg-surface-secondary border border-border rounded-lg">
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2 text-xs text-content-secondary">
                <FolderOpen className="w-4 h-4" />
                <span className="font-mono">{skillsDirectory || "加载中..."}</span>
              </div>
              <button
                onClick={() => {
                  setTempDirectory(skillsDirectory);
                  setEditingDirectory(true);
                }}
                className="p-1 hover:bg-surface-hover rounded text-content-secondary"
                title="修改目录"
              >
                <Edit2 className="w-3 h-3" />
              </button>
            </div>
            <div className="text-2xs text-content-secondary">
              每个 Skill 以独立目录存放，目录名即 Skill ID，包含 SKILL.md 文件
            </div>
          </div>
        )}
      </div>

      {/* Skill 目录列表 */}
      <div>
        <h3 className="text-sm font-medium mb-2">Skill 目录 ({skillDirs.length})</h3>
        {skillDirs.length === 0 ? (
          <div className="text-center py-4 text-xs text-content-secondary border border-dashed border-border rounded-lg">
            <FolderOpen className="w-6 h-6 mx-auto mb-1 opacity-50" />
            <p>还没有 Skill 目录</p>
            <p className="text-2xs mt-1">点击"导入 Skill"创建第一个 Skill 目录</p>
          </div>
        ) : (
          <div className="space-y-1 max-h-48 overflow-y-auto">
            {skillDirs.map((dir) => (
              <div
                key={dir.id}
                className="flex items-center justify-between px-3 py-2 bg-surface-secondary border border-border rounded-md text-xs"
              >
                <div className="flex items-center gap-2 flex-1">
                  <FolderOpen className="w-3 h-3 text-blue-500" />
                  <span className="font-mono font-medium">{dir.id}/</span>
                  <FileText className="w-3 h-3 text-gray-400" />
                  <span className="text-content-secondary">SKILL.md</span>
                  <span className="text-content-secondary">({(dir.file_size / 1024).toFixed(1)} KB)</span>
                </div>
                <div className="flex items-center gap-2">
                  {dir.modified && (
                    <span className="text-content-secondary text-2xs">
                      {new Date(dir.modified * 1000).toLocaleDateString()}
                    </span>
                  )}
                  <span className={`px-1.5 py-0.5 rounded text-2xs ${
                    dir.enabled
                      ? 'bg-green-500/20 text-green-600 dark:text-green-400'
                      : 'bg-gray-500/20 text-gray-600 dark:text-gray-400'
                  }`}>
                    {dir.enabled ? '启用' : '禁用'}
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* 已安装的 Skills */}
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">已加载的 Skills ({skills.length})</h3>
        <div className="flex gap-2">
          <button
            onClick={() => setShowImportModal(true)}
            className="px-3 py-1.5 text-xs bg-primary text-primary-foreground rounded-md hover:bg-primary/90 flex items-center gap-1"
          >
            <Plus className="w-3 h-3" />
            导入 Skill
          </button>
          <button
            onClick={importFromDirectory}
            className="px-3 py-1.5 text-xs bg-surface-secondary border border-border rounded-md hover:bg-surface-hover flex items-center gap-1"
          >
            <FolderOpen className="w-3 h-3" />
            从文件夹导入
          </button>
        </div>
      </div>

      {error && (
        <div className="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md text-xs text-red-600 dark:text-red-400">
          {error}
        </div>
      )}

      {loading ? (
        <div className="text-center py-8 text-sm text-content-secondary">
          加载中...
        </div>
      ) : skills.length === 0 ? (
        <div className="text-center py-8 text-sm text-content-secondary border border-dashed border-border rounded-lg">
          <Code className="w-8 h-8 mx-auto mb-2 opacity-50" />
          <p>还没有安装任何 Skills</p>
          <p className="text-xs mt-1">点击"导入 Skill"开始添加</p>
        </div>
      ) : (
        <div className="space-y-2">
          {skills.map((skill) => (
            <div
              key={skill.id}
              className="p-3 bg-surface-secondary border border-border rounded-lg hover:border-primary/50 transition-colors"
            >
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <h4 className="text-sm font-medium">{skill.name}</h4>
                    <span className="px-2 py-0.5 text-xs bg-blue-500/20 text-blue-600 dark:text-blue-400 rounded-full">
                      {skill.id}
                    </span>
                  </div>
                  <p className="text-xs text-content-secondary mt-1">
                    {skill.description}
                  </p>
                  {skill.tags.length > 0 && (
                    <div className="flex gap-1 mt-2">
                      {skill.tags.map((tag) => (
                        <span
                          key={tag}
                          className="px-1.5 py-0.5 text-xs bg-surface-active rounded"
                        >
                          {tag}
                        </span>
                      ))}
                    </div>
                  )}
                </div>

                <div className="flex items-center gap-2 ml-4">
                  <button
                    onClick={() => toggleSkill(skill.id, !skill.enabled)}
                    className="p-1 hover:bg-surface-hover rounded"
                    title={skill.enabled ? "禁用" : "启用"}
                  >
                    {skill.enabled ? (
                      <ToggleRight className="w-5 h-5 text-green-500" />
                    ) : (
                      <ToggleLeft className="w-5 h-5 text-gray-400" />
                    )}
                  </button>
                  <button
                    onClick={() => previewSkill(skill.id)}
                    className="p-1 hover:bg-surface-hover rounded text-blue-500"
                    title="预览 SKILL.md"
                  >
                    <Eye className="w-4 h-4" />
                  </button>
                  <button
                    onClick={() => deleteSkill(skill.id)}
                    className="p-1 hover:bg-surface-hover rounded text-red-500"
                    title="删除"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Skill 内容预览面板 */}
      {previewSkillId && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
          <div className="w-[700px] max-h-[80vh] bg-surface rounded-xl shadow-2xl border border-border flex flex-col">
            <div className="flex items-center justify-between px-4 py-3 border-b border-border">
              <h3 className="text-sm font-medium">SKILL.md 预览: {previewSkillId}</h3>
              <button
                onClick={closePreview}
                className="p-1 hover:bg-surface-hover rounded"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="flex-1 overflow-y-auto p-4">
              {loadingPreview ? (
                <div className="text-center py-8 text-sm text-content-secondary">
                  加载中...
                </div>
              ) : (
                <pre className="text-xs font-mono whitespace-pre-wrap bg-surface-secondary p-3 rounded-md border border-border">
                  {previewContent}
                </pre>
              )}
            </div>
          </div>
        </div>
      )}

      {/* 导入模态框 */}
      {showImportModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
          <div className="w-[600px] max-h-[80vh] bg-surface rounded-xl shadow-2xl border border-border flex flex-col">
            <div className="flex items-center justify-between px-4 py-3 border-b border-border">
              <h3 className="text-sm font-medium">导入 Skill (Markdown)</h3>
              <button
                onClick={() => setShowImportModal(false)}
                className="p-1 hover:bg-surface-hover rounded"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="flex-1 overflow-y-auto p-4 space-y-4">
              <div>
                <label className="block text-xs font-medium mb-2">
                  SKILL.md 内容
                </label>
                <textarea
                  value={markdownContent}
                  onChange={(e) => setMarkdownContent(e.target.value)}
                  placeholder={`---
name: my-skill
description: 这是一个示例 Skill，用于搜索和总结信息
tags: [search, summary]
---

# My Skill

## 使用说明

当用户需要搜索和总结信息时：
1. 首先调用 web_search 工具搜索关键词
2. 分析搜索结果
3. 使用 summarize_page 工具总结页面内容

## 可用工具

- web_search: 搜索互联网
- open_url: 打开网页
- summarize_page: 总结页面内容`}
                  className="w-full h-64 px-3 py-2 text-xs font-mono bg-surface-secondary border border-border rounded-md focus:outline-none focus:border-primary resize-none"
                />
              </div>

              <div className="text-xs text-content-secondary">
                <p className="font-medium mb-1">SKILL.md 格式说明：</p>
                <ul className="list-disc list-inside space-y-1 text-2xs">
                  <li>使用 YAML frontmatter 定义 Skill 元数据（name, description, tags）</li>
                  <li>正文包含使用说明和步骤，模型会根据这些说明决定如何执行</li>
                  <li>Skill 会作为 tool 暴露给模型，调用时返回完整内容</li>
                  <li>模型根据内容决定调用哪些 MCP tools / 内置工具继续执行</li>
                  <li>每个 Skill 保存为独立目录：skills/skill-name/SKILL.md</li>
                </ul>
              </div>
            </div>

            <div className="px-4 py-3 border-t border-border flex justify-end gap-2">
              <button
                onClick={() => setShowImportModal(false)}
                className="px-3 py-1.5 text-xs border border-border rounded-md hover:bg-surface-hover"
              >
                取消
              </button>
              <button
                onClick={importFromMarkdown}
                disabled={!markdownContent.trim()}
                className="px-3 py-1.5 text-xs bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50"
              >
                导入
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
