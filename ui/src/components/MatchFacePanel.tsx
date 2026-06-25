import { Button, Card, Input } from "@tokimo/ui";
import { Search, UserCheck, UserPlus } from "lucide-react";
import { useState } from "react";
import { api, type MatchFaceResp } from "../api/client";

interface Props {
  t: (key: string) => string;
  onPersonClick?: (personId: string) => void;
}

export function MatchFacePanel({ t, onPersonClick }: Props) {
  const [imageHash, setImageHash] = useState("");
  const [faceIndex, setFaceIndex] = useState("0");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<MatchFaceResp | null>(null);

  const handleMatch = async () => {
    if (!imageHash.trim()) return;
    setLoading(true);
    setError(null);
    setResult(null);
    try {
      const resp = await api.matchFace({
        image_hash: imageHash.trim(),
        face_index: Number(faceIndex) || 0,
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
          <Search size={14} className="text-[var(--color-accent)]" />
          <span className="text-xs font-semibold">{t("matchFace")}</span>
        </div>

        <Input
          value={imageHash}
          onChange={(e) => setImageHash(e.target.value)}
          placeholder={t("imageHash")}
          size="small"
        />

        <Input
          value={faceIndex}
          onChange={(e) => setFaceIndex(e.target.value)}
          placeholder={t("faceIndex")}
          size="small"
          type="number"
        />

        <Button
          size="small"
          onClick={handleMatch}
          disabled={loading || !imageHash.trim()}
          className="cursor-pointer"
        >
          {loading ? t("loading") : t("match")}
        </Button>

        {error && (
          <div className="rounded bg-red-500/10 px-3 py-2 text-xs text-red-500">
            {error}
          </div>
        )}

        {result && (
          <div className="flex flex-col gap-2 rounded border border-black/10 dark:border-white/10 p-3">
            <div className="flex items-center gap-2">
              {result.is_new ? (
                <UserPlus size={14} className="text-orange-500" />
              ) : (
                <UserCheck size={14} className="text-green-500" />
              )}
              <span className="text-xs font-medium">{t("result")}</span>
            </div>
            <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-[11px]">
              <span className="opacity-50">{t("isNew")}</span>
              <span>{result.is_new ? t("yes") : t("no")}</span>
              <span className="opacity-50">{t("similarity")}</span>
              <span>{(result.similarity * 100).toFixed(1)}%</span>
              <span className="opacity-50">{t("person")}</span>
              <span>
                <button
                  type="button"
                  className="cursor-pointer border-none bg-transparent p-0 text-[11px] text-accent-text hover:underline"
                  onClick={() => onPersonClick?.(result.person_id)}
                >
                  {result.person_id}
                </button>
              </span>
            </div>
          </div>
        )}
      </div>
    </Card>
  );
}
