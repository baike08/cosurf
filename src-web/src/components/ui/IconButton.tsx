import { type ReactNode, type ButtonHTMLAttributes, forwardRef } from "react";
import { cn } from "@/lib/utils";

interface IconButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  children: ReactNode;
  size?: "sm" | "md" | "lg";
  variant?: "ghost" | "subtle" | "solid";
  active?: boolean;
}

const sizeMap = {
  sm: "h-6 w-6",
  md: "h-8 w-8",
  lg: "h-10 w-10",
};

const iconSizeMap = {
  sm: "[&_svg]:h-3.5 [&_svg]:w-3.5",
  md: "[&_svg]:h-4 [&_svg]:w-4",
  lg: "[&_svg]:h-5 [&_svg]:w-5",
};

export const IconButton = forwardRef<HTMLButtonElement, IconButtonProps>(
  ({ children, size = "md", variant = "ghost", active, className, ...props }, ref) => {
    return (
      <button
        ref={ref}
        className={cn(
          "inline-flex items-center justify-center rounded-md transition-colors shrink-0",
          "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500",
          "disabled:opacity-40 disabled:pointer-events-none",
          sizeMap[size],
          iconSizeMap[size],
          variant === "ghost" &&
            "text-content-secondary hover:bg-surface-hover hover:text-content",
          variant === "subtle" &&
            "bg-surface-secondary text-content-secondary hover:bg-surface-hover hover:text-content",
          variant === "solid" &&
            "bg-brand-600 text-white hover:bg-brand-700",
          active && "bg-surface-active text-content",
          className,
        )}
        {...props}
      >
        {children}
      </button>
    );
  },
);

IconButton.displayName = "IconButton";
