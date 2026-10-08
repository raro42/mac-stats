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

  // iOS thermal states. English uses the desktop app's words (Nominal / Fair / Serious /
  // Critical) so both apps say the same thing.
  "thermal.title": string;
  /** Spoken and pill label: "Thermal: {level}", like the desktop's thermal card. */
  "thermal.label": string;
  /** What each level means for the iPhone. The chat also sends it to the model, so keep
   *  it about the device (chat effects are in the chat banners). */
  "thermal.meaning.nominal": string;
  "thermal.meaning.fair": string;
  "thermal.meaning.serious": string;
  "thermal.meaning.critical": string;
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
  /** Accessible name of the thermal history strip; `{level}` is the worst state shown. */
  "monitor.thermalHistory": string;
  "monitor.thermalHistoryEmpty": string;
  /** Under the long history views: lines = app open, dots = background samples. */
  "monitor.historyDots": string;
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

  // Used by the chat prompt (Rust reads them from here), not shown in the UI. Labels name
  // RAM and storage unambiguously in each language; the tip uses iOS's real button names.
  "prompt.ram": string;
  "prompt.storage": string;
  "prompt.storageTip": string;
  /** Comma-separated lowercase words that mark a question about this iPhone (battery,
   *  memory, storage, heat…). The chat sends the device data only for those. */
  "prompt.deviceWords": string;
  /** Words that mark a question about storage space (adds the storage tip). Avoid words
   *  that also mean memory, such as German "Speicher". */
  "prompt.spaceWords": string;

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
  "settings.appearance": string;
  /** Default theme that follows iPhone light/dark. Theme names themselves are not translated. */
  "settings.themeSystem": string;
  "settings.history": string;
  "settings.backgroundSamples": string;
  "settings.backgroundNote": string;
  "settings.backgroundOff": string;
  "settings.historyRetention": string;
  "settings.deleteHistory": string;
  "settings.confirmDeleteHistory": string;
  "settings.historyDeleted": string;
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
