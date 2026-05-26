import { type ShellJobEvent, useJobEvents } from "@tokimo/sdk";
import { Button, Card, CircularProgress } from "@tokimo/ui";
import { useCallback, useState } from "react";
import { ButtonRow, fmt, SERVICE, Section, Snapshot } from "./shared";

const BULK_JOB_TYPE = "helloworld_bulk_import";
const LONG_JOB_TYPE = "helloworld_long_running";
type DemoJobKind = "bulk" | "long";
type DemoJobType = typeof BULK_JOB_TYPE | typeof LONG_JOB_TYPE;

interface DemoJobState {
  jobId: string | null;
  status: string;
  progress: number;
  current: number | null;
  total: number | null;
  label: string | null;
  error: string | null;
  lastEvent: unknown;
}

interface ParsedJobEvent {
  jobId: string;
  jobType: DemoJobType;
  status: string;
  progress: number;
  current: number | null;
  total: number | null;
  label: string | null;
  error: string | null;
  raw: unknown;
}

const INITIAL_JOB_STATE: DemoJobState = {
  jobId: null,
  status: "idle",
  progress: 0,
  current: null,
  total: null,
  label: null,
  error: null,
  lastEvent: null,
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function stringField(record: Record<string, unknown> | null, key: string) {
  const value = record?.[key];
  return typeof value === "string" && value.length > 0 ? value : null;
}

function numberField(record: Record<string, unknown> | null, key: string) {
  const value = record?.[key];
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function getJobRecord(event: ShellJobEvent): Record<string, unknown> | null {
  if (isRecord(event.job)) return event.job;
  if (!isRecord(event.data)) return null;
  if (isRecord(event.data.job)) return event.data.job;
  return event.data;
}

function getJobType(job: Record<string, unknown>): DemoJobType | null {
  const value = stringField(job, "type") ?? stringField(job, "kind");
  return value === BULK_JOB_TYPE || value === LONG_JOB_TYPE ? value : null;
}

function getProgressData(job: Record<string, unknown>) {
  const data = isRecord(job.data) ? job.data : null;
  return isRecord(data?.progress) ? data.progress : null;
}

function clampProgress(value: number) {
  return Math.max(0, Math.min(100, Math.round(value)));
}

function parseJobEvent(event: ShellJobEvent): ParsedJobEvent | null {
  if (event.type !== "job_update" && event.type !== "external_job_update")
    return null;
  const job = getJobRecord(event);
  if (!job) return null;

  const jobId = stringField(job, "id");
  const jobType = getJobType(job);
  if (!jobId || !jobType) return null;

  const progressData = getProgressData(job);
  const current = numberField(progressData, "current");
  const total = numberField(progressData, "total");
  const richProgress =
    current !== null && total !== null && total > 0
      ? (current / total) * 100
      : null;
  const rawProgress = numberField(job, "progress") ?? richProgress ?? 0;

  return {
    jobId,
    jobType,
    status: stringField(job, "status") ?? "unknown",
    progress: clampProgress(rawProgress),
    current,
    total,
    label: stringField(progressData, "label"),
    error: stringField(job, "error"),
    raw: event,
  };
}

async function startJob(jobType: DemoJobType, params: Record<string, number>) {
  const res = await fetch(`/api/apps/${SERVICE}/jobs/start`, {
    method: "POST",
    credentials: "include",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ type: jobType, params }),
  });
  if (!res.ok) throw new Error(`${res.status} ${await res.text()}`);
  const body: unknown = await res.json();
  if (!isRecord(body)) throw new Error("Invalid start job response");
  const jobId = stringField(body, "jobId") ?? stringField(body, "job_id");
  if (!jobId) throw new Error("Missing job id in start job response");
  return jobId;
}

function updateFromParsed(parsed: ParsedJobEvent) {
  return (prev: DemoJobState): DemoJobState => ({
    ...prev,
    jobId: parsed.jobId,
    status: parsed.status,
    progress: parsed.progress,
    current: parsed.current,
    total: parsed.total,
    label: parsed.label,
    error: parsed.error,
    lastEvent: parsed.raw,
  });
}

function JobStatusCard({
  title,
  state,
}: {
  title: string;
  state: DemoJobState;
}) {
  const statusText =
    state.status === "completed"
      ? "✓ completed"
      : state.status === "failed"
        ? "✗ failed"
        : state.status;

  return (
    <Card className="flex flex-col gap-3 p-3">
      <div className="flex items-center gap-3">
        <CircularProgress value={state.progress} size={58} />
        <div className="min-w-0 flex-1">
          <div className="text-sm font-medium">{title}</div>
          <div className="truncate text-xs opacity-60">
            {state.jobId ?? "No job yet"}
          </div>
          <div className="text-xs opacity-70">
            {statusText}
            {state.label ? ` · ${state.label}` : ""}
            {state.current !== null && state.total !== null
              ? ` · ${state.current}/${state.total}`
              : ""}
          </div>
        </div>
      </div>
      {state.error && <div className="text-sm text-red-500">{state.error}</div>}
      <Snapshot>
        {fmt({
          jobId: state.jobId,
          status: state.status,
          progress: state.progress,
          current: state.current,
          total: state.total,
          label: state.label,
          error: state.error,
          lastEvent: state.lastEvent,
        })}
      </Snapshot>
    </Card>
  );
}

function useHelloworldJobs(kind: DemoJobKind) {
  const [state, setState] = useState<DemoJobState>(INITIAL_JOB_STATE);
  const [startError, setStartError] = useState<string | null>(null);
  const jobType = kind === "bulk" ? BULK_JOB_TYPE : LONG_JOB_TYPE;

  const applyEvent = useCallback(
    (event: ShellJobEvent) => {
      const parsed = parseJobEvent(event);
      if (!parsed || parsed.jobType !== jobType) return;
      setState((prev) => {
        if (prev.jobId && parsed.jobId !== prev.jobId) return prev;
        return updateFromParsed(parsed)(prev);
      });
    },
    [jobType],
  );

  useJobEvents({ jobTypes: [jobType], onEvent: applyEvent });

  const start = useCallback(async () => {
    setStartError(null);
    const params: Record<string, number> =
      jobType === BULK_JOB_TYPE ? { count: 50 } : { steps: 10, stepMs: 500 };
    try {
      const jobId = await startJob(jobType, params);
      setState({
        ...INITIAL_JOB_STATE,
        jobId,
        status: "queued",
        lastEvent: { jobId, jobType, params },
      });
    } catch (e) {
      setStartError(e instanceof Error ? e.message : String(e));
    }
  }, [jobType]);

  return { state, start, startError };
}

export function BulkImportJobDemo() {
  const { state, start, startError } = useHelloworldJobs("bulk");
  return (
    <Section
      desc="Starts a simulated bulk import job and updates progress from WebSocket job events."
      code="useJobEvents({ jobTypes: ['helloworld_bulk_import'], onEvent })"
    >
      <ButtonRow>
        <Button variant="primary" onClick={start}>
          Start bulk import (50 items)
        </Button>
      </ButtonRow>
      {startError && <div className="text-sm text-red-500">{startError}</div>}
      <JobStatusCard title="Bulk import" state={state} />
    </Section>
  );
}

export function LongRunningJobDemo() {
  const { state, start, startError } = useHelloworldJobs("long");
  return (
    <Section
      desc="Starts a simulated long-running job and renders progress from WebSocket job_update payloads only."
      code="useJobEvents({ jobTypes: ['helloworld_long_running'], onEvent })"
    >
      <ButtonRow>
        <Button variant="primary" onClick={start}>
          Start long-running job (10 steps)
        </Button>
      </ButtonRow>
      {startError && <div className="text-sm text-red-500">{startError}</div>}
      <JobStatusCard title="Long running" state={state} />
    </Section>
  );
}
