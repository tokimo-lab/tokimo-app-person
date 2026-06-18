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
import { Users } from "lucide-react";
import { StrictMode, useState } from "react";
import { createRoot } from "react-dom/client";
import { enUS, zhCN } from "./i18n";
import "./index.css";
import { type PersonDto } from "./api/client";
import { PersonDetail } from "./components/PersonDetail";
import { PersonList } from "./components/PersonList";

type View = "list" | "detail";

function PersonWindow({ ctx }: { ctx: AppRuntimeCtx }) {
  const t = makeTranslator({ "zh-CN": zhCN, "en-US": enUS }, ctx.locale);
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

  return (
    <div className="flex h-full w-full flex-col text-[var(--color-fg-primary)]">
      <header className="flex items-center gap-3 border-b border-black/10 dark:border-white/10 px-4 py-3">
        <Users size={20} style={{ color: "var(--color-accent)" }} />
        <div className="flex flex-col">
          <span className="text-sm font-semibold">{t("title")}</span>
          <span className="text-[10px] opacity-60">{t("subtitle")}</span>
        </div>
      </header>

      <main className="flex-1 overflow-auto p-4">
        {view === "list" && <PersonList t={t} onSelect={handleSelect} />}
        {view === "detail" && selected && (
          <PersonDetail person={selected} t={t} onBack={handleBack} />
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
