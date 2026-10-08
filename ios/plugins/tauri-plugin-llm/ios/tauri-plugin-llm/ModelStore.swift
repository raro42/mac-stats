// GGUF model downloads: progress, resume, SHA-256 verification and free space checks.
//
// One download at a time. The file is downloaded to a temporary folder, verified, and
// only then moved (atomically) to `Application Support/models/`, excluded from the
// iCloud backup.

import CryptoKit
import Foundation
import Tauri

enum ModelStoreError: CodedError {
  case busy
  case notEnoughSpace(needed: Int64, available: Int64)
  case checksumMismatch
  case badResponse(Int)
  case cancelled

  var code: String {
    switch self {
    case .busy: return "download_busy"
    case .notEnoughSpace: return "not_enough_storage"
    case .checksumMismatch: return "checksum_mismatch"
    case .badResponse: return "download_failed"
    case .cancelled: return "download_cancelled"
    }
  }

  var errorDescription: String? {
    switch self {
    case .busy: return "A download is already in progress."
    case .notEnoughSpace(let needed, let available):
      return "Not enough space: \(needed) bytes needed, \(available) available."
    case .checksumMismatch: return "The downloaded file did not match its SHA-256 and was deleted."
    case .badResponse(let code): return "The server answered with status \(code)."
    case .cancelled: return "Download cancelled."
    }
  }
}

struct DownloadRequest {
  let url: URL
  let sha256: String
  let size: Int64
  let file: String
  let allowCellular: Bool
}

final class ModelStore: NSObject, URLSessionDownloadDelegate {
  static func modelsDirectory() -> URL {
    let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
    return base.appendingPathComponent("models", isDirectory: true)
  }

  private var session: URLSession?
  private var task: URLSessionDownloadTask?
  private var request: DownloadRequest?
  private var onEvent: Channel?
  private var completion: ((Result<URL, Error>) -> Void)?
  private var lastProgress = Date.distantPast
  private let lock = NSLock()

  private func resumeDataURL(for file: String) -> URL {
    Self.modelsDirectory().appendingPathComponent(".\(file).resume")
  }

  /// Space available for "important usage" (includes what iOS can purge).
  private func availableSpace() -> Int64 {
    let values = try? Self.modelsDirectory()
      .resourceValues(forKeys: [.volumeAvailableCapacityForImportantUsageKey])
    return values?.volumeAvailableCapacityForImportantUsage ?? 0
  }

  func start(
    _ request: DownloadRequest, onEvent: Channel,
    completion: @escaping (Result<URL, Error>) -> Void
  ) {
    lock.lock()
    defer { lock.unlock() }
    guard task == nil else {
      completion(.failure(ModelStoreError.busy))
      return
    }

    let directory = Self.modelsDirectory()
    try? FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
    let needed = request.size + 1_073_741_824
    let available = availableSpace()
    if available > 0 && available < needed {
      completion(.failure(ModelStoreError.notEnoughSpace(needed: needed, available: available)))
      return
    }

    let config = URLSessionConfiguration.default
    config.allowsCellularAccess = request.allowCellular
    config.timeoutIntervalForRequest = 60
    let session = URLSession(configuration: config, delegate: self, delegateQueue: nil)

    let resumeURL = resumeDataURL(for: request.file)
    let task: URLSessionDownloadTask
    if let data = try? Data(contentsOf: resumeURL) {
      task = session.downloadTask(withResumeData: data)
      try? FileManager.default.removeItem(at: resumeURL)
    } else {
      task = session.downloadTask(with: request.url)
    }

    self.session = session
    self.task = task
    self.request = request
    self.onEvent = onEvent
    self.completion = completion
    task.resume()
  }

  /// Cancels, saving the resume data to continue later.
  func cancel() {
    lock.lock()
    let task = self.task
    let file = request?.file
    lock.unlock()
    task?.cancel(byProducingResumeData: { [weak self] data in
      guard let self, let data, let file else { return }
      try? data.write(to: self.resumeDataURL(for: file))
    })
  }

  private func finish(_ result: Result<URL, Error>) {
    lock.lock()
    let completion = self.completion
    session?.finishTasksAndInvalidate()
    session = nil
    task = nil
    request = nil
    onEvent = nil
    self.completion = nil
    lock.unlock()
    completion?(result)
  }

  // MARK: - URLSessionDownloadDelegate

  func urlSession(
    _ session: URLSession, downloadTask: URLSessionDownloadTask, didWriteData bytesWritten: Int64,
    totalBytesWritten: Int64, totalBytesExpectedToWrite: Int64
  ) {
    guard Date().timeIntervalSince(lastProgress) >= 0.25 else { return }
    lastProgress = Date()
    let total = totalBytesExpectedToWrite > 0 ? totalBytesExpectedToWrite : (request?.size ?? 0)
    onEvent?.send(["type": "progress", "received": Int(totalBytesWritten), "total": Int(total)])
  }

  func urlSession(
    _ session: URLSession, downloadTask: URLSessionDownloadTask,
    didFinishDownloadingTo location: URL
  ) {
    guard let request else { return }
    if let http = downloadTask.response as? HTTPURLResponse, !(200..<300).contains(http.statusCode) {
      finish(.failure(ModelStoreError.badResponse(http.statusCode)))
      return
    }

    // `location` is gone once this method returns: move it before verifying.
    let directory = Self.modelsDirectory()
    let staging = directory.appendingPathComponent(".\(request.file).partial")
    let destination = directory.appendingPathComponent(request.file)
    do {
      try? FileManager.default.removeItem(at: staging)
      try FileManager.default.moveItem(at: location, to: staging)
    } catch {
      finish(.failure(error))
      return
    }

    onEvent?.send(["type": "verifying"] as JsonObject)
    do {
      guard try sha256(of: staging) == request.sha256.lowercased() else {
        try? FileManager.default.removeItem(at: staging)
        finish(.failure(ModelStoreError.checksumMismatch))
        return
      }
      _ = try FileManager.default.replaceItemAt(destination, withItemAt: staging)
      var finalURL = destination
      var values = URLResourceValues()
      values.isExcludedFromBackup = true
      try finalURL.setResourceValues(values)
      finish(.success(finalURL))
    } catch {
      try? FileManager.default.removeItem(at: staging)
      finish(.failure(error))
    }
  }

  func urlSession(_ session: URLSession, task: URLSessionTask, didCompleteWithError error: Error?) {
    guard let error else { return }  // success was already handled in didFinishDownloadingTo
    let cancelled = (error as NSError).code == NSURLErrorCancelled
    finish(.failure(cancelled ? ModelStoreError.cancelled : error))
  }

  // MARK: - Utilities

  private func sha256(of url: URL) throws -> String {
    let handle = try FileHandle(forReadingFrom: url)
    defer { try? handle.close() }
    var hasher = SHA256()
    while let chunk = try handle.read(upToCount: 8 * 1_048_576), !chunk.isEmpty {
      hasher.update(data: chunk)
    }
    return hasher.finalize().map { String(format: "%02x", $0) }.joined()
  }

  static func delete(file: String) throws {
    let directory = modelsDirectory()
    for name in [file, ".\(file).partial", ".\(file).resume"] {
      let url = directory.appendingPathComponent(name)
      if FileManager.default.fileExists(atPath: url.path) {
        try FileManager.default.removeItem(at: url)
      }
    }
  }
}
