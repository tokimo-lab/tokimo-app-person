import { Button, Card, Input } from "@tokimo/ui";
import { Upload } from "lucide-react";
import { useState } from "react";
import { api, type RegisterFacesResp } from "../api/client";

interface Props {
  t: (key: string) => string;
}

export function RegisterFacesPanel({ t }: Props) {
  const [imageHash, setImageHash] = useState("");
  const [sourceApp, setSourceApp] = useState("photo");
  const [sourceId, setSourceId] = useState("");
  const [facesJson, setFacesJson] = useState("[]");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<RegisterFacesResp | null>(null);

  const handleRegister = async () => {
    if (!imageHash.trim() || !sourceId.trim()) return;
    setLoading(true);
    setError(null);
    setResult(null);
    try {
      const parsed = JSON.parse(facesJson);
      const faces = Array.isArray(parsed)
        ? parsed.map((face, index) => {
            const item: Record<string, unknown> =
              face && typeof face === "object" && !Array.isArray(face)
                ? { ...(face as Record<string, unknown>) }
                : { bbox: face };
            if (!Array.isArray(item.embedding)) {
              item.embedding = Array.from({ length: 512 }, (_, i) => (i === index % 512 ? 1 : 0));
            }
            if (typeof item.index !== "number") {
              item.index = index;
            }
            return item;
          })
        : [];
      const resp = await api.registerFaces({
        image_hash: imageHash.trim(),
        source_app: sourceApp.trim(),
        source_id: sourceId.trim(),
        faces,
      });
      setResult(resp);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <Card>
      <div className="flex flex-col gap-3 px-4 py-3">
        <div className="flex items-center gap-2">
          <Upload size={14} className="text-[var(--color-accent)]" />
          <span className="text-xs font-semibold">{t("registerFaces")}</span>
        </div>

        <Input
          value={imageHash}
          onChange={(e) => setImageHash(e.target.value)}
          placeholder={t("imageHash")}
          size="small"
        />

        <Input
          value={sourceApp}
          onChange={(e) => setSourceApp(e.target.value)}
          placeholder={t("sourceApp")}
          size="small"
        />

        <Input
          value={sourceId}
          onChange={(e) => setSourceId(e.target.value)}
          placeholder={t("sourceId")}
          size="small"
        />

        <textarea
          value={facesJson}
          onChange={(e) => setFacesJson(e.target.value)}
          placeholder={t("facesJson")}
          rows={4}
          className="w-full rounded border border-black/10 dark:border-white/10 bg-transparent px-3 py-2 text-xs font-mono resize-y focus:outline-none focus:border-[var(--color-accent)]"
        />

        <Button
          size="small"
          onClick={handleRegister}
          disabled={loading || !imageHash.trim() || !sourceId.trim()}
          className="cursor-pointer"
        >
          {loading ? t("loading") : t("register")}
        </Button>

        {error && (
          <div className="rounded bg-red-500/10 px-3 py-2 text-xs text-red-500">
            {error}
          </div>
        )}

        {result && (
          <div className="flex flex-col gap-1 rounded border border-black/10 dark:border-white/10 p-3 text-[11px]">
            <span className="font-medium">{t("result")}</span>
            <span>
              {t("cached")}: {result.cached}
            </span>
          </div>
        )}
      </div>
    </Card>
  );
}
