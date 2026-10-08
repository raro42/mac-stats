// Inference engine on top of the llama.cpp C API (b11321).
// Follows the pattern of the official example examples/llama.swiftui/llama.cpp.swift/LibLlama.swift.
//
// Every llama.cpp call happens on the plugin queue ("llm.engine"); only
// `requestCancel()` and `snapshot()` are called from other threads, guarded by a lock.

import Foundation
import Tauri
import llama
import os

/// Errors carry a stable `code` that Rust and the web layer translate (see
/// `src/i18n/error-codes.ts`); `errorDescription` is English text for logs only.
enum LlmError: CodedError {
  case notLoaded
  case modelMissing(String)
  case insufficientMemory(needed: UInt64, available: UInt64)
  case loadFailed(String)
  case contextFailed
  case templateFailed
  case tokenizeFailed
  case promptTooLong(tokens: Int, limit: Int)
  case decodeFailed(Int32)

  var code: String {
    switch self {
    case .notLoaded: return "model_not_loaded"
    case .modelMissing: return "model_file_missing"
    case .insufficientMemory: return "not_enough_memory"
    case .loadFailed: return "model_load_failed"
    case .contextFailed: return "context_failed"
    case .templateFailed: return "chat_template_failed"
    case .tokenizeFailed: return "tokenize_failed"
    case .promptTooLong: return "conversation_too_long"
    case .decodeFailed: return "generation_failed"
    }
  }

  var errorDescription: String? {
    switch self {
    case .notLoaded: return "No model is loaded."
    case .modelMissing(let path): return "Model not found at \(path)."
    case .insufficientMemory(let needed, let available):
      return "Not enough memory: \(needed) bytes needed, \(available) available."
    case .loadFailed(let path): return "llama.cpp could not load \(path)."
    case .contextFailed: return "llama.cpp could not create the context."
    case .templateFailed: return "The model's chat template could not be applied."
    case .tokenizeFailed: return "The text could not be tokenized."
    case .promptTooLong(let tokens, let limit):
      return "The conversation takes \(tokens) tokens and the limit is \(limit)."
    case .decodeFailed(let code): return "llama_decode failed (code \(code))."
    }
  }
}

struct LoadOptions {
  var path: String
  var nCtx: UInt32
  var nBatch: UInt32
  var nThreads: Int32
  var gpu: Bool
}

struct ChatTurn {
  let role: String
  let content: String
}

struct GenerateOptions {
  var maxTokens: Int
  var temperature: Float
  var seed: UInt32
  /// Qwen3.5 reasons by default; an empty `<think>` block after the assistant header
  /// turns that off (it is what its official template does without `enable_thinking`).
  var thinkPrefill: Bool
}

/// Memory headroom before iOS kills the app (0 in the simulator).
func availableMemory() -> UInt64 {
  UInt64(os_proc_available_memory())
}

/// Thermal state; debug builds can force it with `IOS_STATS_FAKE_THERMAL`.
private func thermalState() -> ProcessInfo.ThermalState {
  #if DEBUG
    switch ProcessInfo.processInfo.environment["IOS_STATS_FAKE_THERMAL"] {
    case "serious": return .serious
    case "critical": return .critical
    default: break
    }
  #endif
  return ProcessInfo.processInfo.thermalState
}

private func millis(since start: UInt64) -> Double {
  Double(DispatchTime.now().uptimeNanoseconds - start) / 1_000_000
}

final class LlamaEngine {
  private static let backend: Void = { llama_backend_init() }()

  private var model: OpaquePointer?
  private var ctx: OpaquePointer?
  private var vocab: OpaquePointer?
  private var nCtx: UInt32 = 0
  private var nBatch: UInt32 = 512
  /// Tokens currently in the context memory (sequence 0).
  private var cached: [llama_token] = []

  private let lock = NSLock()
  private var cancelRequested = false
  private var loadedPath: String?
  private var busy = false

  // MARK: - State shared across threads

  func requestCancel() {
    lock.lock()
    cancelRequested = true
    lock.unlock()
  }

  private func isCancelled() -> Bool {
    lock.lock()
    defer { lock.unlock() }
    return cancelRequested
  }

  private func begin() {
    lock.lock()
    cancelRequested = false
    busy = true
    lock.unlock()
  }

  private func end() {
    lock.lock()
    busy = false
    lock.unlock()
  }

  func snapshot() -> JsonObject {
    lock.lock()
    defer { lock.unlock() }
    return [
      "loaded": loadedPath != nil,
      "path": loadedPath,
      "busy": busy,
      "availableMemory": Int(availableMemory()),
    ]
  }

  // MARK: - Loading

  func load(_ options: LoadOptions) throws -> JsonObject {
    _ = LlamaEngine.backend
    unload()

    let attrs = try? FileManager.default.attributesOfItem(atPath: options.path)
    let size = (attrs?[.size] as? NSNumber)?.uint64Value ?? 0
    guard size > 0 else { throw LlmError.modelMissing(options.path) }

    let available = availableMemory()
    let needed = size + 512 * 1_048_576
    if available > 0 && available < needed {
      throw LlmError.insufficientMemory(needed: needed, available: available)
    }

    var modelParams = llama_model_default_params()
    #if targetEnvironment(simulator)
      modelParams.n_gpu_layers = 0
    #else
      modelParams.n_gpu_layers = options.gpu ? 99 : 0
    #endif
    modelParams.load_mode = LLAMA_LOAD_MODE_MMAP

    let start = DispatchTime.now().uptimeNanoseconds
    guard let model = llama_model_load_from_file(options.path, modelParams) else {
      throw LlmError.loadFailed(options.path)
    }

    var ctxParams = llama_context_default_params()
    ctxParams.n_ctx = options.nCtx
    ctxParams.n_batch = options.nBatch
    ctxParams.n_ubatch = options.nBatch
    ctxParams.n_threads = options.nThreads
    ctxParams.n_threads_batch = options.nThreads
    ctxParams.flash_attn_type = LLAMA_FLASH_ATTN_TYPE_AUTO
    guard let ctx = llama_init_from_model(model, ctxParams) else {
      llama_model_free(model)
      throw LlmError.contextFailed
    }
    let loadMs = millis(since: start)

    self.model = model
    self.ctx = ctx
    self.vocab = llama_model_get_vocab(model)
    self.nCtx = llama_n_ctx(ctx)
    self.nBatch = options.nBatch
    self.cached = []
    lock.lock()
    loadedPath = options.path
    lock.unlock()

    var desc = [CChar](repeating: 0, count: 256)
    let descLen = llama_model_desc(model, &desc, desc.count)
    let description = descLen > 0 ? String(cString: desc) : ""
    let template = llama_model_chat_template(model, nil).map { String(cString: $0) }

    return [
      "loadMs": loadMs,
      "description": description,
      "nParams": Int(llama_model_n_params(model)),
      "sizeBytes": Int(llama_model_size(model)),
      "nCtx": Int(nCtx),
      "chatTemplate": template,
      "gpu": modelParams.n_gpu_layers > 0,
      "availableMemory": Int(availableMemory()),
    ]
  }

  func unload() {
    if let ctx { llama_free(ctx) }
    if let model { llama_model_free(model) }
    ctx = nil
    model = nil
    vocab = nil
    cached = []
    lock.lock()
    loadedPath = nil
    lock.unlock()
  }

  // MARK: - Text ↔ tokens

  private func applyTemplate(_ turns: [ChatTurn]) throws -> String {
    guard let model else { throw LlmError.notLoaded }
    let roles = turns.map { strdup($0.role) }
    let contents = turns.map { strdup($0.content) }
    defer {
      roles.forEach { free($0) }
      contents.forEach { free($0) }
    }
    var messages = (0..<turns.count).map {
      llama_chat_message(role: UnsafePointer(roles[$0]), content: UnsafePointer(contents[$0]))
    }

    let chars = turns.reduce(0) { $0 + $1.content.utf8.count + $1.role.utf8.count }
    var buffer = [CChar](repeating: 0, count: max(1024, chars * 2 + 256))
    let template = llama_model_chat_template(model, nil)
    func apply(_ buf: inout [CChar]) -> Int32 {
      if let template {
        return llama_chat_apply_template(template, &messages, messages.count, true, &buf, Int32(buf.count))
      }
      return llama_chat_apply_template("chatml", &messages, messages.count, true, &buf, Int32(buf.count))
    }
    var length = apply(&buffer)
    if length > Int32(buffer.count) {
      buffer = [CChar](repeating: 0, count: Int(length) + 1)
      length = apply(&buffer)
    }
    guard length >= 0 else { throw LlmError.templateFailed }
    let bytes = buffer[0..<Int(length)].map { UInt8(bitPattern: $0) }
    return String(decoding: bytes, as: UTF8.self)
  }

  private func tokenize(_ text: String) throws -> [llama_token] {
    guard let vocab else { throw LlmError.notLoaded }
    let length = Int32(text.utf8.count)
    var tokens = [llama_token](repeating: 0, count: Int(length) + 16)
    var count = llama_tokenize(vocab, text, length, &tokens, Int32(tokens.count), true, true)
    if count < 0 {
      tokens = [llama_token](repeating: 0, count: Int(-count))
      count = llama_tokenize(vocab, text, length, &tokens, Int32(tokens.count), true, true)
    }
    guard count >= 0 else { throw LlmError.tokenizeFailed }
    return Array(tokens[0..<Int(count)])
  }

  private func piece(_ token: llama_token) -> [UInt8] {
    guard let vocab else { return [] }
    var buffer = [CChar](repeating: 0, count: 64)
    var count = llama_token_to_piece(vocab, token, &buffer, Int32(buffer.count), 0, false)
    if count < 0 {
      buffer = [CChar](repeating: 0, count: Int(-count))
      count = llama_token_to_piece(vocab, token, &buffer, Int32(buffer.count), 0, false)
    }
    guard count > 0 else { return [] }
    return buffer[0..<Int(count)].map { UInt8(bitPattern: $0) }
  }

  private func makeSampler(_ options: GenerateOptions) -> UnsafeMutablePointer<llama_sampler> {
    let chain = llama_sampler_chain_init(llama_sampler_chain_default_params())!
    llama_sampler_chain_add(
      chain, llama_sampler_init_penalties(llama_vocab_n_tokens(vocab), 64, 1.1, 0, 0))
    if options.temperature <= 0 {
      llama_sampler_chain_add(chain, llama_sampler_init_greedy())
      return chain
    }
    llama_sampler_chain_add(chain, llama_sampler_init_top_k(40))
    llama_sampler_chain_add(chain, llama_sampler_init_top_p(0.9, 1))
    llama_sampler_chain_add(chain, llama_sampler_init_min_p(0.05, 1))
    llama_sampler_chain_add(chain, llama_sampler_init_temp(options.temperature))
    llama_sampler_chain_add(chain, llama_sampler_init_dist(options.seed))
    return chain
  }

  private func decode(_ tokens: inout [llama_token], from start: Int, count: Int) throws {
    guard let ctx else { throw LlmError.notLoaded }
    let rc = tokens.withUnsafeMutableBufferPointer { buffer in
      llama_decode(ctx, llama_batch_get_one(buffer.baseAddress! + start, Int32(count)))
    }
    if rc != 0 { throw LlmError.decodeFailed(rc) }
  }

  // MARK: - Generation

  /// Generates the assistant reply. `emit` receives `{type: "delta", text}` events
  /// batched every ~40 ms.
  func generate(
    turns: [ChatTurn], options: GenerateOptions, emit: (JsonObject) -> Void
  ) throws -> JsonObject {
    guard let ctx, vocab != nil else { throw LlmError.notLoaded }
    begin()
    defer { end() }

    // Earlier replies were generated after the empty <think> block; it is repeated in the
    // history so the prompt continues exactly what is already in memory.
    let think = "<think>\n\n</think>\n\n"
    let rendered = options.thinkPrefill
      ? turns.map { $0.role == "assistant" ? ChatTurn(role: $0.role, content: think + $0.content) : $0 }
      : turns
    var prompt = try applyTemplate(rendered)
    if options.thinkPrefill { prompt += think }
    var tokens = try tokenize(prompt)
    let limit = Int(nCtx) - options.maxTokens
    if tokens.count > limit { throw LlmError.promptTooLong(tokens: tokens.count, limit: limit) }

    // Reuses the prefix shared with what is already in memory. Hybrid models
    // (Qwen3.5, LFM2.5) cannot remove just part of it: then everything is processed.
    var reused = zip(cached, tokens).prefix { $0 == $1 }.count
    if reused == tokens.count { reused -= 1 }  // the last one must be decoded to get logits
    let memory = llama_get_memory(ctx)
    if reused > 0 && !llama_memory_seq_rm(memory, 0, llama_pos(reused), -1) {
      reused = 0
    }
    if reused == 0 { llama_memory_clear(memory, true) }
    cached = Array(tokens[0..<reused])

    // Small chunks: cancellation can only be handled between llama_decode calls,
    // and llama.cpp's abort does not work with Metal.
    let promptChunk = min(Int(nBatch), 64)
    let promptStart = DispatchTime.now().uptimeNanoseconds
    var position = reused
    while position < tokens.count {
      if isCancelled() {
        return ["stopReason": "cancelled", "nPrompt": tokens.count, "nCached": reused, "nGen": 0]
      }
      let count = min(promptChunk, tokens.count - position)
      try decode(&tokens, from: position, count: count)
      cached.append(contentsOf: tokens[position..<(position + count)])
      position += count
    }
    let promptMs = millis(since: promptStart)

    let sampler = makeSampler(options)
    defer { llama_sampler_free(sampler) }

    var pending: [UInt8] = []
    var outgoing = ""
    var lastEmit = DispatchTime.now().uptimeNanoseconds
    var generated = 0
    var stopReason = "length"
    let genStart = DispatchTime.now().uptimeNanoseconds

    generation: while generated < options.maxTokens {
      if isCancelled() {
        stopReason = "cancelled"
        break
      }
      switch thermalState() {
      case .critical:
        stopReason = "thermal"
        break generation
      case .serious:
        usleep(100_000)
      default:
        break
      }

      let token = llama_sampler_sample(sampler, ctx, -1)
      if llama_vocab_is_eog(vocab, token) {
        stopReason = "eos"
        break
      }

      pending.append(contentsOf: piece(token))
      if let text = String(bytes: pending, encoding: .utf8) {
        outgoing += text
        pending.removeAll()
      } else if pending.count > 8 {
        outgoing += String(decoding: pending, as: UTF8.self)
        pending.removeAll()
      }
      if !outgoing.isEmpty && millis(since: lastEmit) >= 40 {
        emit(["type": "delta", "text": outgoing])
        outgoing = ""
        lastEmit = DispatchTime.now().uptimeNanoseconds
      }

      var next = [token]
      try decode(&next, from: 0, count: 1)
      cached.append(token)
      generated += 1
    }

    if !pending.isEmpty { outgoing += String(decoding: pending, as: UTF8.self) }
    if !outgoing.isEmpty { emit(["type": "delta", "text": outgoing]) }
    let genMs = millis(since: genStart)

    return [
      "stopReason": stopReason,
      "nPrompt": tokens.count,
      "nCached": reused,
      "nGen": generated,
      "promptMs": promptMs,
      "genMs": genMs,
      "ppTps": promptMs > 0 ? Double(tokens.count - reused) / (promptMs / 1000) : 0,
      "tgTps": genMs > 0 ? Double(generated) / (genMs / 1000) : 0,
      "availableMemory": Int(availableMemory()),
    ]
  }

  // MARK: - Benchmark (pp = prompt processing, tg = generation)

  func bench(pp: Int, tg: Int, reps: Int) throws -> JsonObject {
    guard let ctx else { throw LlmError.notLoaded }
    begin()
    defer { end() }
    let memory = llama_get_memory(ctx)
    var ppRuns: [Double] = []
    var tgRuns: [Double] = []

    for _ in 0..<max(1, reps) {
      if isCancelled() { break }
      llama_memory_clear(memory, true)
      var prompt = [llama_token](repeating: 100, count: pp)
      let ppStart = DispatchTime.now().uptimeNanoseconds
      var position = 0
      while position < pp {
        let count = min(Int(nBatch), pp - position)
        try decode(&prompt, from: position, count: count)
        position += count
      }
      llama_synchronize(ctx)
      ppRuns.append(Double(pp) / (millis(since: ppStart) / 1000))

      llama_memory_clear(memory, true)
      let tgStart = DispatchTime.now().uptimeNanoseconds
      for _ in 0..<tg {
        var token = [llama_token(100)]
        try decode(&token, from: 0, count: 1)
      }
      llama_synchronize(ctx)
      tgRuns.append(Double(tg) / (millis(since: tgStart) / 1000))
    }

    llama_memory_clear(memory, true)
    cached = []
    let mean = { (xs: [Double]) in xs.isEmpty ? 0 : xs.reduce(0, +) / Double(xs.count) }
    return [
      "ppTps": mean(ppRuns),
      "tgTps": mean(tgRuns),
      "ppRuns": ppRuns,
      "tgRuns": tgRuns,
      "availableMemory": Int(availableMemory()),
    ]
  }
}
