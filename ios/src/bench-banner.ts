// Fixed banner showing the progress of the automatic model benchmark (debug only).
import { listen } from "@tauri-apps/api/event";
import { number, seconds, tokensPerSecond } from "./format";
import { t } from "./i18n";

type BenchEvent = {
  event: string;
  id?: string;
  models?: string[];
  bench?: { tgTps: number };
  item?: { n: number };
  sample?: { t: number; tgTps: number };
};

function describe(e: BenchEvent): string | null {
  const id = e.id ?? "";
  switch (e.event) {
    case "start":
      return t("bench.start", { count: number(e.models?.length ?? 0) });
    case "loaded":
      return t("bench.loaded", { id });
    case "bench":
      return t("bench.speed", { id, speed: tokensPerSecond(e.bench?.tgTps ?? 0) });
    case "quality":
      return t("bench.question", { id, n: number(e.item?.n ?? 0), total: number(10) });
    case "soak":
      return t("bench.soak", {
        id,
        seconds: seconds(e.sample?.t ?? 0),
        speed: tokensPerSecond(e.sample?.tgTps ?? 0),
      });
    case "model_done":
      return t("bench.modelDone");
    case "done":
      return t("bench.done");
    default:
      return null;
  }
}

export async function startBenchBanner(): Promise<void> {
  const banner = document.createElement("div");
  banner.className = "bench-banner";
  banner.hidden = true;
  document.body.append(banner);
  await listen<BenchEvent>("bench-progress", (msg) => {
    const status = describe(msg.payload);
    if (!status) return;
    banner.textContent = msg.payload.event === "done" ? status : t("bench.keepOpen", { status });
    banner.hidden = false;
  });
}
