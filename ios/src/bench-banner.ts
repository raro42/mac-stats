// Aviso fijo con el progreso de la medición automática de modelos (solo depuración).
import { listen } from "@tauri-apps/api/event";

type BenchEvent = {
  event: string;
  id?: string;
  models?: string[];
  bench?: { tgTps: number };
  item?: { n: number };
  sample?: { t: number; tgTps: number };
};

function describe(e: BenchEvent): string {
  switch (e.event) {
    case "start":
      return `Medición de modelos: empezando (${e.models?.length ?? 0})`;
    case "loaded":
      return `Midiendo ${e.id}: modelo cargado`;
    case "bench":
      return `Midiendo ${e.id}: ${e.bench?.tgTps.toFixed(1)} tok/s`;
    case "quality":
      return `Midiendo ${e.id}: pregunta ${e.item?.n}/10`;
    case "soak":
      return `Midiendo ${e.id}: prueba sostenida ${e.sample?.t} s · ${e.sample?.tgTps.toFixed(1)} tok/s`;
    case "model_done":
      return "Modelo terminado";
    case "done":
      return "Medición terminada ✓";
    default:
      return e.event;
  }
}

export async function startBenchBanner(): Promise<void> {
  const banner = document.createElement("div");
  banner.className = "bench-banner";
  banner.hidden = true;
  document.body.append(banner);
  await listen<BenchEvent>("bench-progress", (msg) => {
    banner.textContent = `${describe(msg.payload)} · no cierres la app`;
    if (msg.payload.event === "done") banner.textContent = describe(msg.payload);
    banner.hidden = false;
  });
}
