import { type AppRuntimeCtx, type MenuBarConfig } from "@tokimo/sdk";
import {
  useShellAppearance,
  useShellMenuBar,
  useShellToast,
  useShellWindowNav,
} from "@tokimo/sdk/react";
import { Button } from "@tokimo/ui";
import { useCallback, useEffect, useMemo } from "react";
import { ButtonRow, fmt, Section, Snapshot } from "./shared";

export function CtxDemo({
  ctx,
}: {
  ctx: AppRuntimeCtx;
  t: (k: string) => string;
}) {
  return (
    <Section
      desc="Mount-time context. Captured once at mount; for reactive values prefer the corresponding useShell* hook."
      code="function mount(el, ctx: AppRuntimeCtx) { ... }"
    >
      <Snapshot>
        {fmt({
          windowId: ctx.windowId,
          appId: ctx.appId,
          locale: ctx.locale,
          theme: ctx.theme,
        })}
      </Snapshot>
    </Section>
  );
}

export function AppearanceDemo({ ctx }: { ctx: AppRuntimeCtx }) {
  const ap = useShellAppearance(ctx);
  useEffect(() => {
    console.log("[helloworld] appearance →", ap);
  }, [ap]);
  return (
    <Section
      desc="Reactive theme + title-bar style. Use this instead of ctx.theme so apps re-render when the user toggles dark mode or window chrome."
      code="const ap = useShellAppearance(ctx);"
    >
      <Snapshot>{fmt(ap)}</Snapshot>
    </Section>
  );
}

export function WindowNavDemo({ ctx }: { ctx: AppRuntimeCtx }) {
  const nav = useShellWindowNav(ctx);
  useEffect(() => {
    console.log("[helloworld] windowNav →", {
      route: nav.route,
      canGoBack: nav.canGoBack,
    });
  }, [nav.route, nav.canGoBack]);
  return (
    <Section
      desc="Per-window route stack persisted in DB — F5 restores the route. navigate pushes; replace overwrites; goBack pops."
      code="const nav = useShellWindowNav(ctx);"
    >
      <Snapshot>{fmt({ route: nav.route, canGoBack: nav.canGoBack })}</Snapshot>
      <ButtonRow>
        <Button
          size="small"
          onClick={() => nav.navigate(`/items/${Date.now()}`, "Item")}
        >
          navigate
        </Button>
        <Button size="small" onClick={() => nav.replace("/")}>
          replace /
        </Button>
        <Button
          size="small"
          variant="default"
          disabled={!nav.canGoBack}
          onClick={() => nav.goBack()}
        >
          back
        </Button>
      </ButtonRow>
    </Section>
  );
}

export function ToastDemo({ ctx }: { ctx: AppRuntimeCtx }) {
  const toast = useShellToast(ctx);
  return (
    <Section
      desc="In-window toast notifications (stateless). Four levels: info, success, warning, error."
      code="const toast = useShellToast(ctx); toast.success('hi');"
    >
      <ButtonRow>
        <Button size="small" onClick={() => toast.info("info toast")}>
          info
        </Button>
        <Button size="small" onClick={() => toast.success("success toast")}>
          success
        </Button>
        <Button size="small" onClick={() => toast.warning("warning toast")}>
          warning
        </Button>
        <Button size="small" onClick={() => toast.error("error toast")}>
          error
        </Button>
      </ButtonRow>
    </Section>
  );
}

export function NotifyDemo({
  ctx,
  t,
}: {
  ctx: AppRuntimeCtx;
  t: (k: string) => string;
}) {
  const fire = useCallback(async () => {
    await ctx.shell.notify({
      categoryId: "manual",
      categoryLabel: "helloworld.notifications.manual",
      title: t("notifyTitle"),
      body: t("notifyBody"),
      level: "info",
    });
  }, [ctx.shell, t]);
  return (
    <Section
      desc="Stateless, fire-and-forget. Posts to the notification center which broadcasts via WebSocket. No useShell* wrapper because there's no reactive state."
      code="await ctx.shell.notify({ title, body, level })"
    >
      <ButtonRow>
        <Button size="small" variant="primary" onClick={fire}>
          send notification
        </Button>
      </ButtonRow>
    </Section>
  );
}

export function MenuBarDemo({
  ctx,
  t,
}: {
  ctx: AppRuntimeCtx;
  t: (k: string) => string;
}) {
  const toast = useShellToast(ctx);
  const nav = useShellWindowNav(ctx);
  const config = useMemo<MenuBarConfig>(
    () => ({
      menus: [
        {
          key: "helloworld",
          label: t("menuLabel"),
          items: [
            {
              key: "toast",
              label: t("menuToast"),
              onClick: () => toast.info(t("menuToastMsg")),
            },
            {
              key: "notify",
              label: t("menuNotify"),
              onClick: async () => {
                await ctx.shell.notify({
                  categoryId: "menu",
                  title: t("notifyTitle"),
                  body: t("notifyFromMenu"),
                  level: "info",
                });
              },
            },
            { type: "divider" as const },
            {
              key: "back",
              label: t("menuGoBack"),
              disabled: !nav.canGoBack,
              onClick: () => nav.goBack(),
            },
          ],
        },
      ],
      about: { description: t("subtitle"), version: "0.1.0" },
    }),
    [ctx.shell, nav, toast, t],
  );
  useShellMenuBar(ctx, config);
  return (
    <Section
      desc="Registers the top menubar while this window is focused. Auto-unregisters on unmount."
      code="useShellMenuBar(ctx, config)"
    >
      <p className="text-xs opacity-70">{t("menuHint")}</p>
    </Section>
  );
}
