import type { AppRuntimeCtx } from "@tokimo/sdk";
import { Button, Card, Empty, Input } from "@tokimo/ui";
import { Trash2 } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import type { ItemDto, ItemsListResp } from "../generated/rust-types";
import { SERVICE, Section } from "./shared";

export function ItemsCrudDemo({
  ctx: _ctx,
  t,
}: {
  ctx: AppRuntimeCtx;
  t: (k: string) => string;
}) {
  const [items, setItems] = useState<ItemDto[]>([]);
  const [content, setContent] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const r = await fetch(`/api/apps/${SERVICE}/items`, {
        credentials: "include",
      });
      if (!r.ok) throw new Error(`${r.status} ${await r.text()}`);
      const res = (await r.json()) as ItemsListResp;
      setItems(res.items ?? []);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const add = useCallback(
    async (notify: boolean) => {
      const text = content.trim();
      if (!text) return;
      setError(null);
      try {
        const url = notify
          ? `/api/apps/${SERVICE}/items/notify`
          : `/api/apps/${SERVICE}/items`;
        const r = await fetch(url, {
          method: "POST",
          credentials: "include",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ content: text }),
        });
        if (!r.ok) throw new Error(`${r.status} ${await r.text()}`);
        setContent("");
        await refresh();
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    },
    [content, refresh],
  );

  const remove = useCallback(
    async (id: string) => {
      try {
        const r = await fetch(
          `/api/apps/${SERVICE}/items/${encodeURIComponent(id)}`,
          { method: "DELETE", credentials: "include" },
        );
        if (!r.ok) throw new Error(`${r.status} ${await r.text()}`);
        await refresh();
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    },
    [refresh],
  );

  return (
    <Section
      desc="The original Rust-backed CRUD demo. Validates the multi-process app architecture (Rust handler in src/, accessed via /api/apps/helloworld/*)."
      code="GET/POST/DELETE /api/apps/helloworld/items"
    >
      <Card className="p-3">
        <div className="flex flex-wrap gap-2">
          <Input
            value={content}
            onChange={(e) => setContent(e.target.value)}
            placeholder={t("inputPlaceholder")}
            onKeyDown={(e) => {
              if (e.key === "Enter") void add(false);
            }}
          />
          <Button onClick={() => add(false)}>{t("add")}</Button>
          <Button variant="primary" onClick={() => add(true)}>
            {t("addAndNotify")}
          </Button>
          <Button variant="default" onClick={refresh}>
            {t("refresh")}
          </Button>
        </div>
        {error && (
          <div className="mt-2 text-sm text-red-500">
            {t("error")}
            {error}
          </div>
        )}
      </Card>
      <Card className="p-3">
        {loading ? (
          <div className="opacity-60">{t("loading")}</div>
        ) : items.length === 0 ? (
          <Empty description={t("empty")} />
        ) : (
          <ul className="flex flex-col gap-2">
            {items.map((it) => (
              <li
                key={it.id}
                className="flex items-center justify-between rounded border border-black/10 px-3 py-2 dark:border-white/10"
              >
                <div className="flex flex-col">
                  <span>{it.content}</span>
                  <span className="text-xs opacity-50">{it.created_at}</span>
                </div>
                <Button
                  variant="default"
                  size="small"
                  onClick={() => remove(it.id)}
                >
                  <Trash2 size={14} /> {t("delete")}
                </Button>
              </li>
            ))}
          </ul>
        )}
      </Card>
    </Section>
  );
}
