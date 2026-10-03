// Plugin de Tauri que expone LlamaEngine a Rust (la web nunca lo llama directamente).
//
// Tauri ejecuta todos los comandos de los plugins en una única cola en serie, así que
// cada handler solo valida los argumentos y encola el trabajo en `queue`; `cancel` y
// `status` responden al momento.

import SwiftRs
import Tauri
import UIKit
import WebKit

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

class LlmPlugin: Plugin {
  private let queue = DispatchQueue(label: "llm.engine", qos: .userInitiated)
  private let engine = LlamaEngine()
  private let store = ModelStore()

  override init() {
    super.init()
    let center = NotificationCenter.default
    // Con poca memoria, iOS cierra la app: mejor soltar el modelo antes.
    center.addObserver(
      forName: UIApplication.didReceiveMemoryWarningNotification, object: nil, queue: nil
    ) { [weak self] _ in
      guard let self else { return }
      self.engine.requestCancel()
      self.queue.async { self.engine.unload() }
    }
    // Metal no puede trabajar en segundo plano: se corta la generación al salir.
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
        invoke.reject(error.localizedDescription)
      }
    }
  }

  /// Metal solo puede usar la GPU con la app en primer plano. Si se lanza desde el Mac
  /// con el iPhone en la pantalla de bloqueo, la app arranca en segundo plano.
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
      invoke.reject("La app tiene que estar en pantalla para responder.")
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

  /// Evita que la pantalla se bloquee (pruebas largas o generación en curso).
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
      invoke.reject("La URL del modelo debe ser https.")
      return
    }
    let request = DownloadRequest(
      url: url, sha256: args.sha256, size: args.size, file: args.file,
      allowCellular: args.allowCellular ?? false)
    store.start(request, onEvent: args.onEvent) { result in
      switch result {
      case .success(let file): invoke.resolve(["path": file.path])
      case .failure(let error): invoke.reject(error.localizedDescription)
      }
    }
  }

  @objc public func cancelDownload(_ invoke: Invoke) {
    store.cancel()
    invoke.resolve(["ok": true])
  }

  @objc public func deleteModel(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(DeleteArgs.self)
    // No se puede borrar el archivo que llama.cpp tiene mapeado en memoria.
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
  /// Solo depuración: simula eventos del sistema para el autodiagnóstico.
  @objc public func debugSimulate(_ invoke: Invoke) throws {
    #if DEBUG
      let args = try invoke.parseArgs(SimulateArgs.self)
      let name: Notification.Name
      switch args.event {
      case "memoryWarning": name = UIApplication.didReceiveMemoryWarningNotification
      case "resignActive": name = UIApplication.willResignActiveNotification
      default:
        invoke.reject("Evento desconocido: \(args.event)")
        return
      }
      DispatchQueue.main.async { NotificationCenter.default.post(name: name, object: nil) }
      invoke.resolve(["ok": true])
    #else
      invoke.reject("Solo disponible en builds de depuración.")
    #endif
  }
}

@_cdecl("init_plugin_llm")
func initPlugin() -> Plugin {
  return LlmPlugin()
}
