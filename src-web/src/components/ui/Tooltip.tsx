import { type ReactNode } from "react";
import { cn } from "@/lib/utils";

interface TooltipProps {
  children: ReactNode;
  label: string;
  side?: "top" | "bottom" | "left" | "right";
}

export function Tooltip({ children, label, side = "bottom" }: TooltipProps) {
  const positionClasses = {
    top: "bottom-full left-1/2 -translate-x-1/2 mb-2",
    bottom: "top-full left-1/2 -translate-x-1/2 mt-2",
    left: "right-full top-1/2 -translate-y-1/2 mr-2",
    right: "left-full top-1/2 -translate-y-1/2 ml-2",
  };

  return (
    <div className="relative group inline-flex">
      {children}
      <div
        className={cn(
          "absolute z-50 px-2 py-1 text-xs rounded-md whitespace-nowrap pointer-events-none",
          "bg-content text-content-inverse opacity-0 group-hover:opacity-100 transition-opacity",
          positionClasses[side],
        )}
      >
        {label}
      </div>
    </div>
  );
}
