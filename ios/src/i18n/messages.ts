// Shape of a translation dictionary. Every language file must provide every key, so a
// missing translation is a type error in `pnpm build`.
//
// Placeholders use `{name}` and are filled by `t()` / `tp()`. Plural entries are objects
// keyed by `Intl.PluralRules` category; `other` is always required.
import type { ErrorCode } from "./error-codes";

export type Plural = Partial<Record<Intl.LDMLPluralRule, string>> & { other: string };

type ErrorMessages = { [C in ErrorCode as `error.${C}`]: string };

export interface Messages extends ErrorMessages {
  // Tab bar
  "tabs.label": string;
  "tabs.monitor": string;
  "tabs.chat": string;
  "tabs.settings": string;

  // Shared words
  "common.delete": string;
  "common.cancel": string;
  "common.separator": string;
  /** A measured value in tokens per second, e.g. "18.5 tok/s". */
  "unit.tokensPerSecond": string;

  // Metric names (ring gauges and chart labels)
  "metric.cpu": string;
  "metric.ram": string;
  "metric.storage": string;
  "metric.battery": string;

  // iOS thermal states
  "thermal.nominal": string;
  "thermal.fair": string;
  "thermal.serious": string;
  "thermal.critical": string;
  "thermal.unknown": string;

  // Battery states
  "battery.charging": string;
  "battery.full": string;
  "battery.unplugged": string;

  // Monitor tab
  "monitor.lowPower": string;
  "monitor.readingDevice": string;
  "monitor.gauges": string;
  "monitor.measuring": string;
  /** Accessible name of a gauge that has no data yet. */
  "monitor.gaugeEmpty": string;
  /** Accessible name of a gauge: label, value and optional detail. */
  "monitor.gaugeValue": string;
  "monitor.gaugeValueDetail": string;
  "monitor.appMemory": string;
  "monitor.appMemoryProgress": string;
  "monitor.network": string;
  "monitor.download": string;
  "monitor.upload": string;
  "monitor.history": string;
  "monitor.period": string;
  "monitor.chartCpu": string;
  "monitor.chartRam": string;
  "monitor.footnote": string;
  "monitor.cpuDetail": string;
  "monitor.usedOfTotal": string;
  "monitor.free": string;
  "monitor.noBatterySimulator": string;
  "monitor.unavailable": string;
  "monitor.appMemoryUsed": string;
  "monitor.appMemorySimulator": string;
  "monitor.simulator": string;
  "monitor.osVersion": string;
  "monitor.cores": Plural;
  "monitor.ramTotal": string;
  "monitor.startFailed": string;

  // Device names that need translation
  "device.unknown": string;
  "device.iphoneSe3": string;

  // Chat tab
  "chat.title": string;
  "chat.new": string;
  "chat.tagline": string;
  "chat.empty": string;
  "chat.conversations": string;
  "chat.untitled": string;
  "chat.deleteConversation": string;
  "chat.message": string;
  "chat.placeholder": string;
  "chat.placeholderNoModel": string;
  "chat.send": string;
  "chat.stop": string;
  "chat.pending": string;
  "chat.loadingModel": string;
  "chat.bannerCritical": string;
  "chat.bannerSerious": string;
  "chat.bannerLowPower": string;
  "chat.startFailed": string;

  // Why a reply stopped
  "stop.eos": string;
  "stop.cancelled": string;
  "stop.thermal": string;
  "stop.length": string;
  "stop.error": string;

  // Model card
  "model.title": string;
  "model.option": string;
  "model.optionRecommended": string;
  "model.cellular": string;
  "model.download": string;
  "model.ready": string;
  "model.notInstalled": string;
  "model.preparing": string;
  "model.downloading": string;
  "model.downloadProgress": string;
  "model.verifying": string;
  "model.confirmDelete": string;

  // Settings tab
  "settings.title": string;
  "settings.language": string;
  /** "Automatic" option; `{language}` is the language it resolves to. */
  "settings.automatic": string;
  "settings.languageNote": string;
  "settings.busy": string;

  // Model lab (debug builds only)
  "lab.title": string;
  "lab.model": string;
  "lab.load": string;
  "lab.bench": string;
  "lab.unload": string;
  "lab.preparing": string;
  "lab.defaultPrompt": string;
  "lab.generate": string;
  "lab.notInstalled": string;
  "lab.pickModel": string;
  "lab.noModels": string;
  "lab.loading": string;
  "lab.loaded": string;
  "lab.unloaded": string;
  "lab.benching": string;
  "lab.benchResult": string;
  "lab.generateResult": string;
  "lab.tokens": Plural;
  "lab.reused": Plural;
  "lab.error": string;
  "lab.startFailed": string;

  // Automatic model benchmark banner (debug builds only)
  "bench.start": string;
  "bench.loaded": string;
  "bench.speed": string;
  "bench.question": string;
  "bench.soak": string;
  "bench.modelDone": string;
  "bench.done": string;
  "bench.keepOpen": string;
}
