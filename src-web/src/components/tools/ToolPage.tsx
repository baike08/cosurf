import { JsonParser, JsonEditor, JsonValidator } from "./JsonTools";
import { Wrench } from "lucide-react";

interface ToolPageProps {
  toolId: string;
}

export function ToolPage({ toolId }: ToolPageProps) {
  return (
    <div className="h-full flex flex-col">
      {(() => {
        switch (toolId) {
          case "json-parser":
            return <JsonParser />;
          case "json-editor":
            return <JsonEditor />;
          case "json-validator":
            return <JsonValidator />;
          default:
            return (
              <div className="h-full flex flex-col items-center justify-center gap-4 text-content-tertiary bg-surface-secondary pt-[80px]">
                <Wrench className="w-12 h-12" />
                <div className="text-center">
                  <div className="text-sm font-medium text-content-secondary">
                    工具开发中
                  </div>
                  <div className="text-xs mt-1">
                    「{toolId}」功能即将推出
                  </div>
                </div>
              </div>
            );
        }
      })()}
    </div>
  );
}

/**
 * 解析 cosurf://tools/ URL，返回工具 ID
 * 例如: cosurf://tools/json-parser -> "json-parser"
 */
export function parseToolUrl(url: string): string | null {
  try {
    if (!url.startsWith("cosurf://tools/")) return null;
    const toolId = url.replace("cosurf://tools/", "").split("?")[0]?.split("#")[0];
    return toolId || null;
  } catch {
    return null;
  }
}

/**
 * 判断 URL 是否是工具箱内部页面
 */
export function isToolUrl(url: string): boolean {
  return url.startsWith("cosurf://tools/");
}
