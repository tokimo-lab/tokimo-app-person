import {
  type AppRuntimeCtx,
  type Dispose,
  defineApp,
  makeTranslator,
  type MenuBarConfig,
  RuntimeProvider,
  useMenuBar,
} from "@tokimo/sdk";
import {
  ConfigProvider,
  ToastProvider,
  enUS as uiEnUS,
  zhCN as uiZhCN,
} from "@tokimo/ui";
import { FlaskConical, Users } from "lucide-react";
import { StrictMode, useEffect, useMemo, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { enUS, zhCN } from "./i18n";
import "./index.css";
import { type PersonDto } from "./api/client";
import { PersonDebugPanel } from "./components/PersonDebugPanel";
import { PersonDetail } from "./components/PersonDetail";
import { PersonList } from "./components/PersonList";

type View = "list" | "detail";

function PersonWindow({ ctx }: { ctx: AppRuntimeCtx }) {
  const t = useMemo(
    () => makeTranslator({ "zh-CN": zhCN, "en-US": enUS }, ctx.locale),
    [ctx.locale],
  );
  const [view, setView] = useState<View>("list");
  const [selected, setSelected] = useState<PersonDto | null>(null);
  const [debugOpen, setDebugOpen] = useState(false);
  const [personEventsVersion, setPersonEventsVersion] = useState(0);
  const seenPersonEventIdsRef = useRef<Set<string>>(new Set());

  useEffect(() => {
    return ctx.shell.appEntityEvents.subscribe({
      appId: "person",
      kind: "person",
      onEvent: (event) => {
        const payload = event.payload;
        const eventId =
          typeof payload === "object" &&
          payload !== null &&
          typeof (payload as { eventId?: unknown }).eventId === "string"
            ? (payload as { eventId: string }).eventId
            : null;
        if (eventId) {
          const seen = seenPersonEventIdsRef.current;
          if (seen.has(eventId)) return;
          seen.add(eventId);
          if (seen.size > 256) {
            const first = seen.values().next().value;
            if (typeof first === "string") seen.delete(first);
          }
        }
        setPersonEventsVersion((value) => value + 1);
      },
    });
  }, [ctx.shell.appEntityEvents]);

  const menuBarConfig = useMemo<MenuBarConfig>(
    () => ({
      appMenu: [
        {
          key: "person-debug-tools",
          label: t("debugMenu"),
          icon: <FlaskConical size={14} />,
          onClick: () => setDebugOpen(true),
        },
      ],
    }),
    [t],
  );
  useMenuBar(menuBarConfig);

  const handleSelect = (person: PersonDto) => {
    setSelected(person);
    setView("detail");
  };

  const handleBack = () => {
    setView("list");
    setSelected(null);
  };

  const handlePersonClick = (personId: string) => {
    setDebugOpen(false);
    handleSelect({
      id: personId,
      name: null,
      avatar_url: null,
      face_count: 0,
      media_count: 0,
      created_at: "",
      updated_at: "",
    });
  };

  return (
    <div className="relative flex h-full w-full flex-col bg-surface-base text-fg-primary">
      <header className="flex items-center gap-3 border-b border-base px-4 py-3">
        <Users size={20} className="text-accent-text" />
        <div className="flex flex-col">
          <span className="text-sm font-semibold">{t("title")}</span>
          <span className="text-xs text-fg-secondary">{t("subtitle")}</span>
        </div>
      </header>

      <main className="flex-1 overflow-auto p-4">
        {view === "list" && (
          <PersonList
            t={t}
            onSelect={handleSelect}
            onDebugOpen={() => setDebugOpen(true)}
            refreshToken={personEventsVersion}
          />
        )}
        {view === "detail" && selected && (
          <PersonDetail
            person={selected}
            t={t}
            onBack={handleBack}
            refreshToken={personEventsVersion}
          />
        )}
      </main>

      <PersonDebugPanel
        open={debugOpen}
        t={t}
        onClose={() => setDebugOpen(false)}
        onPersonClick={handlePersonClick}
      />
    </div>
  );
}

export default defineApp({
  id: "person",
  manifest: {
    id: "person",
    appName: "Person",
    icon: "Users",
    color: "#8b5cf6",
    windowType: "person",
    defaultSize: { width: 900, height: 700 },
    category: "app",
  },
  translations: { "zh-CN": zhCN, "en-US": enUS },
  mount(container, ctx): Dispose {
    const root = createRoot(container);
    const locale = ctx.locale.startsWith("zh") ? uiZhCN : uiEnUS;
    root.render(
      <StrictMode>
        <ConfigProvider locale={locale}>
          <ToastProvider>
            <RuntimeProvider value={ctx}>
              <PersonWindow ctx={ctx} />
            </RuntimeProvider>
          </ToastProvider>
        </ConfigProvider>
      </StrictMode>,
    );
    return () => root.unmount();
  },
});
