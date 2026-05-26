import type { AppRuntimeCtx } from "@tokimo/sdk";
import type { ReactNode } from "react";

export const SERVICE = "helloworld";

export function fmt(v: unknown): string {
  try {
    return JSON.stringify(
      v,
      (_k, val) => {
        if (typeof val === "function") return "[fn]";
        if (val instanceof Element) return "[Element]";
        return val;
      },
      2,
    );
  } catch {
    return String(v);
  }
}

export interface DemoEntry {
  id: string;
  category: string;
  title: string;
  api: string;
  Component: React.FC<{ ctx: AppRuntimeCtx; t: (k: string) => string }>;
}

export function Section({
  desc,
  code,
  children,
}: {
  desc: string;
  code: string;
  children: ReactNode;
}) {
  return (
    <div className="flex flex-col gap-3">
      <p className="text-sm opacity-80 leading-relaxed">{desc}</p>
      <pre className="rounded bg-black/[0.06] dark:bg-white/[0.06] px-3 py-2 text-xs font-mono">
        {code}
      </pre>
      {children}
    </div>
  );
}

export function Snapshot({ children }: { children: ReactNode }) {
  return (
    <pre className="rounded border border-black/10 bg-white/60 dark:border-white/10 dark:bg-black/40 px-3 py-2 text-xs leading-snug whitespace-pre-wrap break-words">
      {children}
    </pre>
  );
}

export function ButtonRow({ children }: { children: ReactNode }) {
  return <div className="flex flex-wrap gap-2">{children}</div>;
}
