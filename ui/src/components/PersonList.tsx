import { Card, Empty, Input } from "@tokimo/ui";
import { Search, Users } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { api, type PersonDto } from "../api/client";

interface Props {
  t: (key: string) => string;
  onSelect: (person: PersonDto) => void;
}

export function PersonList({ t, onSelect }: Props) {
  const [persons, setPersons] = useState<PersonDto[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [offset, setOffset] = useState(0);
  const limit = 20;

  useEffect(() => {
    setLoading(true);
    setError(null);
    api
      .listPersons({ limit, offset })
      .then((resp) => {
        setPersons(resp.items);
        setTotal(resp.total);
      })
      .catch((e) => setError(e instanceof Error ? e.message : String(e)))
      .finally(() => setLoading(false));
  }, [offset]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return persons;
    return persons.filter((p) => p.name.toLowerCase().includes(q));
  }, [persons, query]);

  const hasMore = offset + limit < total;

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12 text-sm opacity-50">
        {t("loading")}
      </div>
    );
  }

  if (error) {
    return (
      <div className="rounded bg-red-500/10 px-3 py-2 text-sm text-red-500">
        {t("error")}
        {error}
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-3">
      <div className="relative">
        <Search
          size={12}
          className="pointer-events-none absolute left-2 top-1/2 -translate-y-1/2 opacity-50 z-10"
        />
        <Input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder={t("searchPlaceholder")}
          className="w-full pl-7"
          size="small"
        />
      </div>

      {filtered.length === 0 ? (
        <Empty description={t("noPersons")} />
      ) : (
        <div className="flex flex-col gap-2">
          {filtered.map((person) => (
            <Card
              key={person.id}
              className="cursor-pointer transition hover:border-[var(--color-accent)] hover:shadow-sm"
              onClick={() => onSelect(person)}
            >
              <div className="flex items-center gap-3 px-4 py-3">
                <div className="flex h-9 w-9 items-center justify-center rounded-full bg-[var(--color-accent-subtle)]">
                  <Users size={16} className="text-[var(--color-accent)]" />
                </div>
                <div className="flex flex-1 flex-col">
                  <span className="text-sm font-medium">
                    {person.name || t("unnamed")}
                  </span>
                  <span className="text-[11px] opacity-60">
                    {person.face_count} {t("faceCount")}
                  </span>
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}

      {(offset > 0 || hasMore) && (
        <div className="flex items-center justify-between pt-2">
          <button
            type="button"
            disabled={offset === 0}
            onClick={() => setOffset(Math.max(0, offset - limit))}
            className="cursor-pointer rounded px-3 py-1.5 text-xs transition hover:bg-black/[0.05] dark:hover:bg-white/[0.05] disabled:opacity-40"
          >
            ← {t("back")}
          </button>
          <span className="text-[11px] opacity-50">
            {offset + 1}–{Math.min(offset + limit, total)} / {total}
          </span>
          <button
            type="button"
            disabled={!hasMore}
            onClick={() => setOffset(offset + limit)}
            className="cursor-pointer rounded px-3 py-1.5 text-xs transition hover:bg-black/[0.05] dark:hover:bg-white/[0.05] disabled:opacity-40"
          >
            {t("detail")} →
          </button>
        </div>
      )}
    </div>
  );
}
