import {
  AppSetupGuide,
  Card,
  Empty,
  Input,
  type AppSetupGuideProps,
} from "@tokimo/ui";
import { Database, FlaskConical, Image, Search, Users } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { api, type PersonDto } from "../api/client";

type GuideIcon = AppSetupGuideProps["features"][number]["icon"];

const guideIcon = (icon: typeof Users) => icon as unknown as GuideIcon;

interface Props {
  t: (key: string) => string;
  onSelect: (person: PersonDto) => void;
  onDebugOpen: () => void;
}

export function PersonList({ t, onSelect, onDebugOpen }: Props) {
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
    return persons.filter((person) => {
      const name = person.name?.toLowerCase() ?? t("unnamed").toLowerCase();
      return name.includes(q);
    });
  }, [persons, query, t]);

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

  if (total === 0 && query.trim() === "") {
    return (
      <AppSetupGuide
        imageSrc="/api/apps/person/assets/icon.png"
        accentColor="purple"
        title={t("setupTitle")}
        description={t("setupDescription")}
        features={[
          { icon: guideIcon(Users), label: t("setupFeatureIdentity") },
          { icon: guideIcon(Image), label: t("setupFeatureSources") },
          { icon: guideIcon(Database), label: t("setupFeatureOwnership") },
        ]}
        actionLabel={t("setupAction")}
        actionIcon={guideIcon(FlaskConical)}
        onAction={onDebugOpen}
        className="-m-4 h-[calc(100%+2rem)]"
      />
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-1">
        <span className="text-base font-semibold">{t("peopleLibrary")}</span>
        <span className="text-xs text-fg-secondary">{t("peopleIntro")}</span>
      </div>

      <div className="relative max-w-md">
        <Search
          size={12}
          className="pointer-events-none absolute left-2 top-1/2 z-10 -translate-y-1/2 text-fg-muted"
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
        <div className="grid grid-cols-[repeat(auto-fill,minmax(160px,1fr))] gap-3">
          {filtered.map((person) => (
            <Card
              key={person.id}
              className="cursor-pointer overflow-hidden border-base bg-surface-raised transition hover:border-accent hover:shadow-sm"
              onClick={() => onSelect(person)}
            >
              <div className="flex flex-col gap-3 p-3">
                <div className="relative aspect-square overflow-hidden rounded-md bg-accent-subtle">
                  {person.avatar_url ? (
                    <img
                      src={person.avatar_url}
                      alt={person.name ?? t("unnamed")}
                      className="h-full w-full object-cover"
                      loading="lazy"
                    />
                  ) : (
                    <div className="flex h-full w-full items-center justify-center">
                      <Users size={34} className="text-accent-text" />
                    </div>
                  )}
                </div>
                <div className="flex min-w-0 flex-col gap-1">
                  <span className="truncate text-sm font-semibold">
                    {person.name || t("unnamed")}
                  </span>
                  <div className="flex items-center gap-3 text-xs text-fg-secondary">
                    <span className="flex items-center gap-1">
                      <Users size={12} />
                      {person.face_count}
                    </span>
                    <span className="flex items-center gap-1">
                      <Image size={12} />
                      {person.media_count}
                    </span>
                  </div>
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
            className="cursor-pointer rounded px-3 py-1.5 text-xs text-fg-secondary transition hover:bg-fill-secondary disabled:opacity-40"
          >
            ← {t("back")}
          </button>
          <span className="text-xs text-fg-muted">
            {offset + 1}–{Math.min(offset + limit, total)} / {total}
          </span>
          <button
            type="button"
            disabled={!hasMore}
            onClick={() => setOffset(offset + limit)}
            className="cursor-pointer rounded px-3 py-1.5 text-xs text-fg-secondary transition hover:bg-fill-secondary disabled:opacity-40"
          >
            {t("detail")} →
          </button>
        </div>
      )}
    </div>
  );
}
