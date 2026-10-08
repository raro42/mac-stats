// Tauri plugin that exposes LlamaEngine to Rust (the web layer never calls it directly).
//
// Tauri runs every plugin command on a single serial queue, so each handler only
// validates its arguments and enqueues the work on `queue`; `cancel` and `status`
// reply immediately.

import NaturalLanguage
import SwiftRs
import Tauri
import UIKit
import WebKit

/// An error with a stable code from `src/i18n/error-codes.ts`. The web layer shows a
/// translated message for the code; the English description only goes to logs.
protocol CodedError: LocalizedError {
  var code: String { get }
}

/// Code for any error thrown by the engine, the model store, URLSession or FileManager.
func errorCode(_ error: Error) -> String {
  if let coded = error as? CodedError { return coded.code }
  if let url = error as? URLError {
    if url.networkUnavailableReason == .cellular { return "cellular_not_allowed" }
    switch url.code {
    case .notConnectedToInternet, .networkConnectionLost, .dataNotAllowed: return "network_offline"
    case .timedOut: return "network_timeout"
    default: return "network"
    }
  }
  if error is CocoaError || (error as NSError).domain == NSPOSIXErrorDomain { return "storage" }
  return "internal"
}

extension Invoke {
  func reject(_ error: Error) {
    reject(error.localizedDescription, code: errorCode(error))
  }
}

class LoadArgs: Decodable {
  let path: String
  var nCtx: UInt32?
  var nBatch: UInt32?
  var nThreads: Int32?
  var gpu: Bool?
}

class BenchArgs: Decodable {
  var pp: Int?
  var tg: Int?
  var reps: Int?
}

class TurnArgs: Decodable {
  let role: String
  let content: String
}

class GenerateArgs: Decodable {
  let messages: [TurnArgs]
  var maxTokens: Int?
  var temperature: Float?
  var seed: UInt32?
  var thinkPrefill: Bool?
  let onEvent: Channel
}

class KeepAwakeArgs: Decodable {
  let enabled: Bool
}

class DownloadArgs: Decodable {
  let url: String
  let sha256: String
  let size: Int64
  let file: String
  var allowCellular: Bool?
  let onEvent: Channel
}

class DeleteArgs: Decodable {
  let file: String
}

class SimulateArgs: Decodable {
  let event: String
}

class DetectLanguageArgs: Decodable {
  let text: String
}

class BackgroundRefreshArgs: Decodable {
  let enabled: Bool
}

class LlmPlugin: Plugin {
  private let queue = DispatchQueue(label: "llm.engine", qos: .userInitiated)
  private let engine = LlamaEngine()
  private let store = ModelStore()

  override init() {
    super.init()
    let center = NotificationCenter.default
    // When memory runs low, iOS kills the app: better to release the model first.
    center.addObserver(
      forName: UIApplication.didReceiveMemoryWarningNotification, object: nil, queue: nil
    ) { [weak self] _ in
      guard let self else { return }
      self.engine.requestCancel()
      self.queue.async { self.engine.unload() }
    }
    // Metal cannot work in the background: generation stops when the app leaves the foreground.
    center.addObserver(
      forName: UIApplication.willResignActiveNotification, object: nil, queue: nil
    ) { [weak self] _ in
      self?.engine.requestCancel()
    }
  }

  private func run(_ invoke: Invoke, _ work: @escaping () throws -> JsonObject) {
    queue.async {
      do {
        invoke.resolve(try work())
      } catch {
        invoke.reject(error)
      }
    }
  }

  /// Metal can only use the GPU while the app is in the foreground. When launched from the
  /// Mac with the iPhone on the lock screen, the app starts in the background.
  private func isActive() -> Bool {
    if Thread.isMainThread { return UIApplication.shared.applicationState == .active }
    return DispatchQueue.main.sync { UIApplication.shared.applicationState == .active }
  }

  @objc public func status(_ invoke: Invoke) {
    var status = engine.snapshot()
    status["active"] = isActive()
    invoke.resolve(status)
  }

  @objc public func load(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(LoadArgs.self)
    let options = LoadOptions(
      path: args.path,
      nCtx: args.nCtx ?? 4096,
      nBatch: args.nBatch ?? 512,
      nThreads: args.nThreads ?? 2,
      gpu: args.gpu ?? true
    )
    run(invoke) { try self.engine.load(options) }
  }

  @objc public func unload(_ invoke: Invoke) {
    engine.requestCancel()
    run(invoke) {
      self.engine.unload()
      return ["ok": true]
    }
  }

  @objc public func bench(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(BenchArgs.self)
    run(invoke) {
      try self.engine.bench(pp: args.pp ?? 512, tg: args.tg ?? 128, reps: args.reps ?? 3)
    }
  }

  @objc public func generate(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(GenerateArgs.self)
    guard isActive() else {
      invoke.reject("The app must be in the foreground to generate.", code: "app_in_background")
      return
    }
    let turns = args.messages.map { ChatTurn(role: $0.role, content: $0.content) }
    let options = GenerateOptions(
      maxTokens: args.maxTokens ?? 512,
      temperature: args.temperature ?? 0.7,
      seed: args.seed ?? UInt32.random(in: 1...UInt32.max),
      thinkPrefill: args.thinkPrefill ?? false
    )
    let channel = args.onEvent
    run(invoke) {
      try self.engine.generate(turns: turns, options: options) { event in
        channel.send(event)
      }
    }
  }

  @objc public func cancel(_ invoke: Invoke) {
    engine.requestCancel()
    invoke.resolve(["ok": true])
  }

  /// Keeps the screen from locking (long tests or a generation in progress).
  @objc public func keepAwake(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(KeepAwakeArgs.self)
    DispatchQueue.main.async {
      UIApplication.shared.isIdleTimerDisabled = args.enabled
    }
    invoke.resolve(["ok": true])
  }
}

extension LlmPlugin {
  @objc public func download(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(DownloadArgs.self)
    guard let url = URL(string: args.url), url.scheme == "https" else {
      invoke.reject("The model URL must use https.", code: "insecure_url")
      return
    }
    let request = DownloadRequest(
      url: url, sha256: args.sha256, size: args.size, file: args.file,
      allowCellular: args.allowCellular ?? false)
    store.start(request, onEvent: args.onEvent) { result in
      switch result {
      case .success(let file): invoke.resolve(["path": file.path])
      case .failure(let error): invoke.reject(error)
      }
    }
  }

  @objc public func cancelDownload(_ invoke: Invoke) {
    store.cancel()
    invoke.resolve(["ok": true])
  }

  @objc public func deleteModel(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(DeleteArgs.self)
    // The file that llama.cpp has memory-mapped cannot be deleted.
    run(invoke) {
      if (self.engine.snapshot()["path"] as? String)?.hasSuffix("/\(args.file)") == true {
        self.engine.unload()
      }
      try ModelStore.delete(file: args.file)
      return ["ok": true]
    }
  }
}

extension LlmPlugin {
  /// Language of a piece of text, detected on the device with NaturalLanguage (no network).
  /// Replies `{language: {code, englishName, confidence}}`, or `{language: null}` when
  /// nothing is recognized. `code` is BCP-47 (`es`, `en`, `zh-Hans`, …).
  @objc public func detectLanguage(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(DetectLanguageArgs.self)
    let recognizer = NLLanguageRecognizer()
    recognizer.processString(args.text)
    guard
      let (language, confidence) = recognizer.languageHypotheses(withMaximum: 1)
        .max(by: { $0.value < $1.value }),
      language != .undetermined
    else {
      invoke.resolve(["language": NSNull()])
      return
    }
    let code = language.rawValue
    let name = Locale(identifier: "en").localizedString(forIdentifier: code) ?? code
    invoke.resolve([
      "language": ["code": code, "englishName": name, "confidence": confidence] as JsonObject
    ])
  }

  /// Turns the history's Background App Refresh wake-ups on or off (BackgroundRefresh.swift).
  @objc public func setBackgroundRefresh(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(BackgroundRefreshArgs.self)
    BackgroundRefresh.enabled = args.enabled
    if args.enabled { BackgroundRefresh.schedule() } else { BackgroundRefresh.cancel() }
    invoke.resolve(["ok": true])
  }

  /// Whether iOS allows Background App Refresh for the app (the user can turn it off).
  @objc public func backgroundRefreshStatus(_ invoke: Invoke) {
    invoke.resolve(["status": BackgroundRefresh.status()])
  }

  /// Debug only: simulates system events for the self-test.
  @objc public func debugSimulate(_ invoke: Invoke) throws {
    #if DEBUG
      let args = try invoke.parseArgs(SimulateArgs.self)
      let name: Notification.Name
      switch args.event {
      case "memoryWarning": name = UIApplication.didReceiveMemoryWarningNotification
      case "resignActive": name = UIApplication.willResignActiveNotification
      default:
        invoke.reject("Unknown event: \(args.event)")
        return
      }
      DispatchQueue.main.async { NotificationCenter.default.post(name: name, object: nil) }
      invoke.resolve(["ok": true])
    #else
      invoke.reject("Only available in debug builds.")
    #endif
  }
}

@_cdecl("init_plugin_llm")
func initPlugin() -> Plugin {
  return LlmPlugin()
}
