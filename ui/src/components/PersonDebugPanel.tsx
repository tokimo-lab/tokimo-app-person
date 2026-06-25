import { Button } from "@tokimo/ui";
import { X } from "lucide-react";
import { MatchFacePanel } from "./MatchFacePanel";
import { RegisterFacesPanel } from "./RegisterFacesPanel";

interface Props {
  open: boolean;
  t: (key: string) => string;
  onClose: () => void;
  onPersonClick: (personId: string) => void;
}

export function PersonDebugPanel({
  open,
  t,
  onClose,
  onPersonClick,
}: Props) {
  if (!open) return null;

  return (
    <div className="absolute inset-0 z-20 flex items-center justify-center bg-black/20 p-5 backdrop-blur-sm">
      <div className="flex max-h-full w-full max-w-4xl flex-col overflow-hidden rounded-lg border border-base bg-surface-overlay text-fg-primary shadow-lg">
        <div className="flex items-center gap-3 border-b border-base px-4 py-3">
          <div className="flex min-w-0 flex-1 flex-col">
            <span className="text-sm font-semibold">{t("debugTitle")}</span>
            <span className="text-xs text-fg-secondary">
              {t("debugDescription")}
            </span>
          </div>
          <Button
            size="small"
            variant="text"
            onClick={onClose}
            aria-label={t("close")}
            className="cursor-pointer"
          >
            <X size={14} />
          </Button>
        </div>

        <div className="grid gap-4 overflow-auto p-4 md:grid-cols-2">
          <MatchFacePanel t={t} onPersonClick={onPersonClick} />
          <RegisterFacesPanel t={t} />
        </div>
      </div>
    </div>
  );
}
