"use client";

import { CircleCheckIcon, InfoIcon, Loader2Icon, OctagonXIcon, TriangleAlertIcon } from "lucide-react";
import { useTheme } from "next-themes";
import { Toaster as Sonner, type ToasterProps } from "sonner";

const Toaster = ({ ...props }: ToasterProps) => {
  const { theme = "system" } = useTheme();

  return (
    <Sonner
      theme={theme as ToasterProps["theme"]}
      className="toaster group"
      closeButton
      icons={{
        success: <CircleCheckIcon className="size-4" />,
        info: <InfoIcon className="size-4" />,
        warning: <TriangleAlertIcon className="size-4" />,
        error: <OctagonXIcon className="size-4" />,
        loading: <Loader2Icon className="size-4 animate-spin" />,
      }}
      toastOptions={{
        classNames: {
          toast: "bg-card/95 backdrop-blur-xl border-border/50 shadow-2xl dither",
          title: "text-foreground",
          description: "text-muted-foreground",
          success: "bg-card/95 backdrop-blur-xl border-border/50 text-foreground",
          error: "bg-card/95 backdrop-blur-xl border-red-500/30 text-foreground",
          warning: "bg-card/95 backdrop-blur-xl border-yellow-500/30 text-foreground",
          info: "bg-card/95 backdrop-blur-xl border-border/50 text-foreground",
          closeButton:
            "bg-muted/80 border-border/50 text-foreground hover:bg-muted hover:border-border transition-all",
        },
      }}
      style={
        {
          "--normal-bg": "hsl(var(--card) / 0.95)",
          "--normal-text": "hsl(var(--foreground))",
          "--normal-border": "hsl(var(--border) / 0.5)",
          "--success-bg": "hsl(var(--card) / 0.95)",
          "--success-text": "hsl(var(--foreground))",
          "--success-border": "hsl(var(--border) / 0.5)",
          "--error-bg": "hsl(var(--card) / 0.95)",
          "--error-text": "hsl(var(--foreground))",
          "--error-border": "hsl(0 63% 50% / 0.3)",
          "--warning-bg": "hsl(var(--card) / 0.95)",
          "--warning-text": "hsl(var(--foreground))",
          "--warning-border": "hsl(45 93% 47% / 0.3)",
          "--info-bg": "hsl(var(--card) / 0.95)",
          "--info-text": "hsl(var(--foreground))",
          "--info-border": "hsl(var(--border) / 0.5)",
          "--border-radius": "var(--radius)",
        } as React.CSSProperties
      }
      {...props}
    />
  );
};

export { Toaster };
