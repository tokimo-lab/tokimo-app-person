import { Button, Card, Input } from "@tokimo/ui";
import { ArrowLeft, Image, Pencil, Save, Users, X } from "lucide-react";
import { useEffect, useState } from "react";
import {
  api,
  fetchPersonPhotos,
  type PersonDetailDto,
  type PersonDto,
  type PhotoOutput,
} from "../api/client";

interface Props {
  person: PersonDto;
  t: (key: string) => string;
  onBack: () => void;
}

export function PersonDetail({ person, t, onBack }: Props) {
  const [detail, setDetail] = useState<PersonDetailDto | null>(null);
  const [photos, setPhotos] = useState<PhotoOutput[]>([]);
  const [loading, setLoading] = useState(true);
  const [photosLoading, setPhotosLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState(false);
  const [editName, setEditName] = useState(person.name);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    setLoading(true);
    setError(null);
    api
      .getPerson(person.id)
      .then(setDetail)
      .catch((e) => setError(e instanceof Error ? e.message : String(e)))
      .finally(() => setLoading(false));

    // Fetch photos from photo app
    setPhotosLoading(true);
    fetchPersonPhotos(person.id)
      .then(setPhotos)
      .catch(() => setPhotos([]))
      .finally(() => setPhotosLoading(false));
  }, [person.id]);

  const handleSave = async () => {
    if (!editName.trim()) return;
    setSaving(true);
    try {
      const updated = await api.updatePerson(person.id, {
        name: editName.trim(),
      });
      setDetail(updated);
      setEditing(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
    }
  };

  const handleCancel = () => {
    setEditing(false);
    setEditName(detail?.name ?? person.name);
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
  };

  return (
    <div className="flex flex-col gap-4">
      <button
        type="button"
        onClick={onBack}
        className="cursor-pointer flex items-center gap-1 text-xs opacity-60 hover:opacity-100 transition"
      >
        <ArrowLeft size={14} />
        {t("back")}
      </button>

      {/* Person info card */}
      <Card>
        <div className="flex items-center gap-3 px-4 py-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-full bg-[var(--color-accent-subtle)]">
            <Users size={18} className="text-[var(--color-accent)]" />
          </div>
          <div className="flex flex-1 flex-col">
            {editing ? (
              <div className="flex items-center gap-2">
                <Input
                  value={editName}
                  onChange={(e) => setEditName(e.target.value)}
                  size="small"
                  className="flex-1"
                  onKeyDown={(e) => e.key === "Enter" && handleSave()}
                />
                <Button
                  size="small"
                  onClick={handleSave}
                  disabled={saving || !editName.trim()}
                >
                  <Save size={12} />
                </Button>
                <Button size="small" variant="text" onClick={handleCancel}>
                  <X size={12} />
                </Button>
              </div>
            ) : (
              <div className="flex items-center gap-2">
                <span className="text-sm font-semibold">
                  {data.name || t("unnamed")}
                </span>
                <button
                  type="button"
                  onClick={() => {
                    setEditing(true);
                    setEditName(data.name ?? "");
                  }}
                  className="cursor-pointer opacity-40 hover:opacity-100 transition"
                  title={t("edit")}
                >
                  <Pencil size={12} />
                </button>
              </div>
            )}
            <span className="text-[11px] opacity-60">
              {data.face_count} {t("faceCount")}
              {photos.length > 0 && ` · ${photos.length} ${t("mediaCount")}`}
            </span>
          </div>
        </div>
      </Card>

      {/* Photos from photo app */}
      <section className="flex flex-col gap-2">
        <h3 className="text-xs font-semibold uppercase tracking-wide opacity-50">
          {t("mediaCount")} ({photos.length})
        </h3>
        {photosLoading ? (
          <div className="text-xs opacity-40">{t("loading")}</div>
        ) : photos.length === 0 ? (
          <div className="text-xs opacity-40">{t("noResult")}</div>
        ) : (
          <div className="grid grid-cols-[repeat(auto-fill,minmax(100px,1fr))] gap-2">
            {photos.map((photo) => (
              <div
                key={photo.id}
                className="flex flex-col items-center gap-1 rounded border border-black/10 dark:border-white/10 p-2"
              >
                <img
                  src={`/api/thumb/photo/${photo.id}?w=200&h=200`}
                  alt={photo.filename}
                  className="h-16 w-16 rounded object-cover"
                  loading="lazy"
                />
                <span
                  className="max-w-full truncate text-[10px] opacity-50"
                  title={photo.filename}
                >
                  {photo.filename}
                </span>
              </div>
            ))}
          </div>
        )}
      </section>

      {/* Face detections */}
      <section className="flex flex-col gap-2">
        <h3 className="text-xs font-semibold uppercase tracking-wide opacity-50">
          {t("faceCount")} ({data.faces.length})
        </h3>
        {data.faces.length === 0 ? (
          <div className="text-xs opacity-40">{t("noResult")}</div>
        ) : (
          <div className="grid grid-cols-[repeat(auto-fill,minmax(80px,1fr))] gap-2">
            {data.faces.map((face) => (
              <div
                key={face.id}
                className="flex flex-col items-center gap-1 rounded border border-black/10 dark:border-white/10 p-2"
              >
                <img
                  src={`/api/thumb/photo/${face.image_hash}?w=200&h=200`}
                  alt=""
                  className="h-14 w-14 rounded object-cover"
                  loading="lazy"
                />
                <span className="text-[10px] opacity-50">
                  {face.source_app}
                </span>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
