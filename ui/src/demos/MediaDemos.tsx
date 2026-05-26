import type { AppRuntimeCtx } from "@tokimo/sdk";
import { useMediaCenter } from "@tokimo/sdk/react";
import { Button, Empty } from "@tokimo/ui";
import { useEffect } from "react";
import { ButtonRow, fmt, Section, Snapshot } from "./shared";

export function MediaCenterSnapshotDemo({ ctx }: { ctx: AppRuntimeCtx }) {
  const { snapshot } = useMediaCenter(ctx);
  useEffect(() => {
    console.log("[helloworld] media.snapshot →", snapshot);
  }, [snapshot]);
  return (
    <Section
      desc="Central media center snapshot. Reactive — re-renders whenever the active provider's playback state changes. `null` = no provider playing."
      code="const { snapshot } = useMediaCenter(ctx); snapshot?.isPlaying;"
    >
      <Snapshot>
        {fmt(
          snapshot
            ? {
                providerId: snapshot.providerId,
                isPlaying: snapshot.isPlaying,
                currentTimeMs: snapshot.currentTimeMs,
                durationMs: snapshot.durationMs,
                volume: snapshot.volume,
                shuffle: snapshot.shuffle,
                repeatMode: snapshot.repeatMode,
                currentIndex: snapshot.currentIndex,
                queueLen: snapshot.queue.length,
              }
            : null,
        )}
      </Snapshot>
    </Section>
  );
}

export function MediaSessionDemo({ ctx }: { ctx: AppRuntimeCtx }) {
  const { snapshot, api } = useMediaCenter(ctx);

  if (!api) {
    return (
      <Section
        desc="Playback controls for the active media session."
        code="const { snapshot, api } = useMediaCenter(ctx);"
      >
        <Empty description="Media API unavailable in this host environment." />
      </Section>
    );
  }

  const currentTrack =
    snapshot && snapshot.queue[snapshot.currentIndex] != null
      ? snapshot.queue[snapshot.currentIndex]
      : null;

  const handleSeek = (deltaMs: number) => {
    if (!snapshot) return;
    const target = snapshot.currentTimeMs + deltaMs;
    const clamped =
      snapshot.durationMs > 0
        ? Math.max(0, Math.min(target, snapshot.durationMs))
        : Math.max(0, target);
    api.seek(clamped);
  };

  const handleVolume = (delta: number) => {
    if (!snapshot) return;
    api.setVolume(Math.max(0, Math.min(1, snapshot.volume + delta)));
  };

  return (
    <Section
      desc="Interactive controls for the active media session. pause/resume, next/prev, seek ±5 s, volume ±0.1."
      code="const { snapshot, api } = useMediaCenter(ctx);"
    >
      {snapshot == null ? (
        <Empty description="No active media source." />
      ) : (
        <Snapshot>
          {fmt({
            providerId: snapshot.providerId,
            title: currentTrack?.title ?? null,
            artist: currentTrack?.artist ?? null,
          })}
        </Snapshot>
      )}
      <ButtonRow>
        <Button
          size="small"
          disabled={snapshot == null}
          onClick={() => api.previous()}
        >
          ⏮ prev
        </Button>
        <Button
          size="small"
          disabled={snapshot == null}
          onClick={() => {
            if (!snapshot) return;
            snapshot.isPlaying ? api.pause() : api.resume();
          }}
        >
          {snapshot?.isPlaying ? "⏸ pause" : "▶ play"}
        </Button>
        <Button
          size="small"
          disabled={snapshot == null}
          onClick={() => api.next()}
        >
          ⏭ next
        </Button>
        <Button
          size="small"
          disabled={snapshot == null}
          onClick={() => handleSeek(-5000)}
        >
          ⏪ -5s
        </Button>
        <Button
          size="small"
          disabled={snapshot == null}
          onClick={() => handleSeek(5000)}
        >
          +5s ⏩
        </Button>
        <Button
          size="small"
          disabled={snapshot == null}
          onClick={() => handleVolume(-0.1)}
        >
          🔉 vol-
        </Button>
        <Button
          size="small"
          disabled={snapshot == null}
          onClick={() => handleVolume(0.1)}
        >
          🔊 vol+
        </Button>
      </ButtonRow>
    </Section>
  );
}
