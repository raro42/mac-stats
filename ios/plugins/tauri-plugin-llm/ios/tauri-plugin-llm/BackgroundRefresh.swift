// Background App Refresh for the saved history.
//
// iOS wakes the app from time to time (it decides when, roughly every 15 min to a few
// hours depending on use). While the app runs, the Rust sampler takes its usual one-second
// readings and saves them as background minutes. This file only registers the task, keeps
// the app awake for a short sampling window, and schedules the next wake-up. The LLM is
// never loaded here.
//
// iOS requires the task to be registered before the app finishes launching, earlier than
// Tauri loads plugins, so `ios_stats_register_background_refresh` is called from the app's
// `main.mm` before the app starts.

import BackgroundTasks
import Foundation
import UIKit

enum BackgroundRefresh {
  static let identifier = "com.gilberto.iosstats.refresh"
  /// Mirror of the app setting (Rust stores the real one), readable before Rust runs.
  private static let enabledKey = "backgroundHistoryEnabled"
  /// How long each wake-up samples before telling iOS it is done (budget is ~30 s).
  private static let sampleSeconds: TimeInterval = 12
  /// Earliest next wake-up; iOS usually waits longer.
  private static let interval: TimeInterval = 15 * 60

  static var enabled: Bool {
    get { UserDefaults.standard.object(forKey: enabledKey) as? Bool ?? true }
    set { UserDefaults.standard.set(newValue, forKey: enabledKey) }
  }

  static func register() {
    let registered = BGTaskScheduler.shared.register(forTaskWithIdentifier: identifier, using: nil) { task in
      guard let task = task as? BGAppRefreshTask else {
        task.setTaskCompleted(success: false)
        return
      }
      handle(task)
    }
    NSLog("iOS Stats: background refresh registered: %@", registered ? "yes" : "no")
    NotificationCenter.default.addObserver(
      forName: UIApplication.didEnterBackgroundNotification, object: nil, queue: nil
    ) { _ in schedule() }
  }

  static func schedule() {
    guard enabled else { return }
    let request = BGAppRefreshTaskRequest(identifier: identifier)
    request.earliestBeginDate = Date(timeIntervalSinceNow: interval)
    do {
      try BGTaskScheduler.shared.submit(request)
    } catch {
      NSLog("iOS Stats: background refresh not scheduled: %@", error.localizedDescription)
    }
  }

  static func cancel() {
    BGTaskScheduler.shared.cancel(taskRequestWithIdentifier: identifier)
  }

  private static func handle(_ task: BGAppRefreshTask) {
    schedule()
    let lock = NSLock()
    var finished = false
    let finish = { (success: Bool) in
      lock.lock()
      defer { lock.unlock() }
      guard !finished else { return }
      finished = true
      task.setTaskCompleted(success: success)
    }
    task.expirationHandler = { finish(false) }
    DispatchQueue.main.asyncAfter(deadline: .now() + sampleSeconds) { finish(true) }
  }

  /// "available", "denied" (the user turned Background App Refresh off) or "restricted".
  static func status() -> String {
    let read = { () -> String in
      switch UIApplication.shared.backgroundRefreshStatus {
      case .available: return "available"
      case .denied: return "denied"
      case .restricted: return "restricted"
      @unknown default: return "unknown"
      }
    }
    return Thread.isMainThread ? read() : DispatchQueue.main.sync(execute: read)
  }
}

@_cdecl("ios_stats_register_background_refresh")
public func registerBackgroundRefresh() {
  BackgroundRefresh.register()
}
