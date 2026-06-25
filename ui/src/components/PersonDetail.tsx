import { Button, Card, Input } from "@tokimo/ui";
import {
  ArrowLeft,
  Database,
  Image,
  Pencil,
  Save,
  UserRound,
  Users,
  X,
} from "lucide-react";
import { useEffect, useState } from "react";
import {
  api,
  type FaceDetailDto,
  type PersonDetailDto,
  type PersonDto,
  type SourceMediaDto,
} from "../api/client";
import { getSourceLabel, getSourceThumbnailUrl } from "../lib/sourcePreview";

interface Props {
  person: PersonDto;
  t: (key: string) => string;
  onBack: () => void;
}

export function PersonDetail({ person, t, onBack }: Props) {
  const [detail, setDetail] = useState<PersonDetailDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState(false);
  const [editName, setEditName] = useState(person.name ?? "");
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    setLoading(true);
    setError(null);
    api
      .getPerson(person.id)
      .then(setDetail)
      .catch((e) => setError(e instanceof Error ? e.message : String(e)))
      .finally(() => setLoading(false));
  }, [person.id]);

  const handleSave = async () => {
    if (!editName.trim()) return;
    setSaving(true);
    try {
      const updated = await api.updatePerson(person.id, {
        name: editName.trim(),
      });
      setDetail((current) => (current ? { ...current, ...updated } : null));
      setEditing(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
    }
  };

  const handleCancel = () => {
    setEditing(false);
    setEditName(detail?.name ?? person.name ?? "");
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12 text-sm opacity-50">
        {t("loading")}
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col gap-3">
        <button
          type="button"
          onClick={onBack}
          className="cursor-pointer flex items-center gap-1 text-xs opacity-60 hover:opacity-100 transition"
        >
          <ArrowLeft size={14} />
          {t("back")}
        </button>
        <div className="rounded bg-red-500/10 px-3 py-2 text-sm text-red-500">
          {t("error")}
          {error}
        </div>
      </div>
    );
  }

  const data = detail ?? {
    ...person,
    faces: [],
    media: [],
  };
  const displayName = data.name || t("unnamed");
  const previewSource = data.faces[0] ?? data.media[0] ?? null;
  const avatarUrl =
    data.avatar_url ??
    (previewSource
      ? getSourceThumbnailUrl(
          {
            sourceApp: previewSource.source_app,
            sourceId: previewSource.source_id,
          },
          320,
        )
      : null);
  const media = buildMediaList(data.media, data.faces);

  return (
    <div className="flex flex-col gap-4">
      <button
        type="button"
        onClick={onBack}
        className="flex cursor-pointer items-center gap-1 text-xs text-fg-secondary transition hover:text-fg-primary"
      >
        <ArrowLeft size={14} />
        {t("back")}
      </button>

      <Card>
        <div className="flex flex-col gap-4 p-4 sm:flex-row sm:items-center">
          <div className="h-28 w-28 overflow-hidden rounded-lg bg-accent-subtle">
            {avatarUrl ? (
              <img
                src={avatarUrl}
                alt={displayName}
                className="h-full w-full object-cover"
                loading="lazy"
              />
            ) : (
              <div className="flex h-full w-full items-center justify-center">
                <UserRound size={44} className="text-accent-text" />
              </div>
            )}
          </div>

          <div className="flex min-w-0 flex-1 flex-col gap-3">
            {editing ? (
              <div className="flex items-center gap-2">
                <Input
                  value={editName}
                  onChange={(e) => setEditName(e.target.value)}
                  size="small"
                  className="min-w-0 flex-1"
                  onKeyDown={(e) => e.key === "Enter" && handleSave()}
                  autoFocus
                />
                <Button
                  size="small"
                  onClick={handleSave}
                  disabled={saving || !editName.trim()}
                  aria-label={t("save")}
                  className="cursor-pointer"
                >
                  <Save size={12} />
                </Button>
                <Button
                  size="small"
                  variant="text"
                  onClick={handleCancel}
                  aria-label={t("cancel")}
                  className="cursor-pointer"
                >
                  <X size={12} />
                </Button>
              </div>
            ) : (
              <div className="flex items-center gap-2">
                <span className="min-w-0 truncate text-lg font-semibold">
                  {displayName}
                </span>
                <button
                  type="button"
                  onClick={() => {
                    setEditing(true);
                    setEditName(data.name ?? "");
                  }}
                  className="cursor-pointer rounded p-1 text-fg-muted transition hover:bg-fill-secondary hover:text-fg-primary"
                  title={t("edit")}
                >
                  <Pencil size={14} />
                </button>
              </div>
            )}

            <div className="flex flex-wrap gap-2 text-xs text-fg-secondary">
              <span className="flex items-center gap-1 rounded bg-fill-secondary px-2 py-1">
                <Users size={12} />
                {data.face_count} {t("faceCount")}
              </span>
              <span className="flex items-center gap-1 rounded bg-fill-secondary px-2 py-1">
                <Image size={12} />
                {data.media_count || media.length} {t("mediaCount")}
              </span>
            </div>
          </div>
        </div>
      </Card>

      <section className="flex flex-col gap-2">
        <div className="flex items-center gap-2 text-sm font-semibold">
          <Image size={15} className="text-accent-text" />
          {t("linkedSources")}
        </div>
        {media.length === 0 ? (
          <div className="rounded border border-dashed border-base px-3 py-6 text-center text-xs text-fg-muted">
            {t("noLinkedSources")}
          </div>
        ) : (
          <div className="grid grid-cols-[repeat(auto-fill,minmax(110px,1fr))] gap-2">
            {media.map((item) => (
              <SourceTile key={item.key} item={item} />
            ))}
          </div>
        )}
      </section>

      <section className="flex flex-col gap-2">
        <div className="flex items-center gap-2 text-sm font-semibold">
          <Users size={15} className="text-accent-text" />
          {t("faceSamples")}
        </div>
        {data.faces.length === 0 ? (
          <div className="rounded border border-dashed border-base px-3 py-6 text-center text-xs text-fg-muted">
            {t("noResult")}
          </div>
        ) : (
          <div className="grid grid-cols-[repeat(auto-fill,minmax(96px,1fr))] gap-2">
            {data.faces.map((face) => (
              <div
                key={face.id}
                className="flex min-w-0 flex-col gap-1 rounded-md border border-base bg-surface-raised p-2"
              >
                <div className="aspect-square overflow-hidden rounded bg-fill-secondary">
                  <PreviewImage
                    source={face}
                    alt={getSourceLabel(face.source_app)}
                  />
                </div>
                <span className="truncate text-xs text-fg-secondary">
                  {getSourceLabel(face.source_app)} #{face.face_index + 1}
                </span>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}

type DisplayMedia = {
  key: string;
  source_app: string;
  source_id: string;
};

function buildMediaList(
  media: SourceMediaDto[],
  faces: FaceDetailDto[],
): DisplayMedia[] {
  if (media.length > 0) {
    return media.map((item) => ({
      key: item.id,
      source_app: item.source_app,
      source_id: item.source_id,
    }));
  }

  const seen = new Set<string>();
  const items: DisplayMedia[] = [];
  for (const face of faces) {
    const key = `${face.source_app}:${face.source_id}`;
    if (seen.has(key)) continue;
    seen.add(key);
    items.push({
      key,
      source_app: face.source_app,
      source_id: face.source_id,
    });
  }
  return items;
}

function SourceTile({ item }: { item: DisplayMedia }) {
  return (
    <div className="flex min-w-0 flex-col gap-1 rounded-md border border-base bg-surface-raised p-2">
      <div className="aspect-square overflow-hidden rounded bg-fill-secondary">
        <PreviewImage source={item} alt={getSourceLabel(item.source_app)} />
      </div>
      <div className="flex min-w-0 items-center gap-1 text-xs text-fg-secondary">
        <Database size={12} />
        <span className="truncate">{getSourceLabel(item.source_app)}</span>
      </div>
    </div>
  );
}

function PreviewImage({
  source,
  alt,
}: {
  source: { source_app: string; source_id: string };
  alt: string;
}) {
  const thumbnailUrl = getSourceThumbnailUrl(
    {
      sourceApp: source.source_app,
      sourceId: source.source_id,
    },
    240,
  );

  if (!thumbnailUrl) {
    return (
      <div className="flex h-full w-full items-center justify-center">
        <Image size={22} className="text-fg-muted" />
      </div>
    );
  }

  return (
    <img
      src={thumbnailUrl}
      alt={alt}
      className="h-full w-full object-cover"
      loading="lazy"
    />
  );
}
