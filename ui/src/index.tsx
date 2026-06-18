import {
  type AppRuntimeCtx,
  type Dispose,
  defineApp,
  makeTranslator,
  RuntimeProvider,
} from "@tokimo/sdk";
import {
  ConfigProvider,
  ToastProvider,
  enUS as uiEnUS,
  zhCN as uiZhCN,
} from "@tokimo/ui";
import { FlaskConical, Users } from "lucide-react";
import { StrictMode, useState } from "react";
import { createRoot } from "react-dom/client";
import { enUS, zhCN } from "./i18n";
import "./index.css";
import { type PersonDto } from "./api/client";
import { MatchFacePanel } from "./components/MatchFacePanel";
import { PersonDetail } from "./components/PersonDetail";
import { PersonList } from "./components/PersonList";
import { RegisterFacesPanel } from "./components/RegisterFacesPanel";

type View = "list" | "detail";
type Tab = "persons" | "test";

function PersonWindow({ ctx }: { ctx: AppRuntimeCtx }) {
  const t = makeTranslator({ "zh-CN": zhCN, "en-US": enUS }, ctx.locale);
  const [tab, setTab] = useState<Tab>("persons");
  const [view, setView] = useState<View>("list");
  const [selected, setSelected] = useState<PersonDto | null>(null);

  const handleSelect = (person: PersonDto) => {
    setSelected(person);
    setView("detail");
  };

  const handleBack = () => {
    setView("list");
    setSelected(null);
  };

  const handlePersonClick = (personId: string) => {
    setTab("persons");
    handleSelect({
      id: personId,
      name: "",
      face_count: 0,
      media_count: 0,
      created_at: "",
      updated_at: "",
    });
  };

  return (
    <div className="flex h-full w-full flex-col text-[var(--color-fg-primary)]">
      <header className="flex items-center gap-3 border-b border-black/10 dark:border-white/10 px-4 py-3">
        <Users size={20} style={{ color: "var(--color-accent)" }} />
        <div className="flex flex-col">
          <span className="text-sm font-semibold">{t("title")}</span>
          <span className="text-[10px] opacity-60">{t("subtitle")}</span>
        </div>
        <div className="flex-1" />
        <div className="flex gap-1">
          <button
            type="button"
            className={`cursor-pointer rounded px-2.5 py-1 text-[11px] transition ${
              tab === "persons"
                ? "bg-[var(--color-accent)] text-white"
                : "opacity-60 hover:opacity-100"
            }`}
            onClick={() => setTab("persons")}
          >
            {t("persons")}
          </button>
          <button
            type="button"
            className={`cursor-pointer rounded px-2.5 py-1 text-[11px] transition flex items-center gap-1 ${
              tab === "test"
                ? "bg-[var(--color-accent)] text-white"
                : "opacity-60 hover:opacity-100"
            }`}
            onClick={() => setTab("test")}
          >
            <FlaskConical size={12} />
            {t("testTab")}
          </button>
        </div>
      </header>

      <main className="flex-1 overflow-auto p-4">
        {tab === "persons" && (
          <>
            {view === "list" && <PersonList t={t} onSelect={handleSelect} />}
            {view === "detail" && selected && (
              <PersonDetail person={selected} t={t} onBack={handleBack} />
            )}
          </>
        )}
        {tab === "test" && (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <MatchFacePanel t={t} onPersonClick={handlePersonClick} />
            <RegisterFacesPanel t={t} />
          </div>
        )}
      </main>
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
