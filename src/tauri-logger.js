//! Tauri Console Logger
//! Intercepts console.log/warn/error and forwards to Tauri Rust logs
//! Include this file before other scripts to enable automatic logging

(function setupTauriConsoleLogger() {
  const hasInternalsInvoke =
    typeof window.__TAURI_INTERNALS__ === 'object' &&
    typeof window.__TAURI_INTERNALS__.invoke === 'function';
  // Only set up if Tauri is available
  if (
    typeof window.__TAURI__ === 'undefined' &&
    typeof window.__TAURI_INVOKE__ === 'undefined' &&
    !hasInternalsInvoke
  ) {
    return; // Tauri not available, skip setup
  }
  
  const originalLog = console.log;
  const originalWarn = console.warn;
  const originalError = console.error;
  const originalDebug = console.debug;
  const originalInfo = console.info;
  
  function getInvoke() {
    if (typeof window.__TAURI__ !== 'undefined' && window.__TAURI__.core?.invoke) {
      return window.__TAURI__.core.invoke;
    }
    const internals = window.__TAURI_INTERNALS__;
    if (internals && typeof internals.invoke === 'function') {
      return internals.invoke.bind(internals);
    }
    if (typeof window.__TAURI_INVOKE__ !== 'undefined') {
      return window.__TAURI_INVOKE__;
    }
    return null;
  }
  
  function forwardToTauri(level, args, source = 'js') {
    try {
      const invokeFn = getInvoke();
      if (!invokeFn) {
        return; // Tauri not ready yet
      }
      
      // Convert arguments to string
      const message = Array.from(args)
        .map(arg => {
          if (typeof arg === 'object') {
            try {
              return JSON.stringify(arg, null, 2);
            } catch {
              return String(arg);
            }
          }
          return String(arg);
        })
        .join(' ');
      
      // Forward to Tauri (non-blocking, don't wait for response)
      invokeFn('log_from_js', {
        level: level,
        message: message,
        source: source
      }).catch(() => {
        // Silently fail if Tauri isn't ready or command doesn't exist
      });
    } catch (err) {
      // Silently fail - don't break console if logging fails
    }
  }
  
  // Get source from stack trace or use default
  function getSource() {
    try {
      const stack = new Error().stack;
      if (stack) {
        const lines = stack.split('\n');
        // Try to find the calling file name
        for (let i = 2; i < lines.length && i < 5; i++) {
          const line = lines[i];
          const match = line.match(/([^/\\]+\.js)/);
          if (match) {
            return match[1];
          }
        }
      }
    } catch {
      // Ignore errors in source detection
    }
    return 'js';
  }
  
  // Forward warn/error only. Idle metrics refresh used to spam console.log →
  // log_from_js IPC every tick and keep tauri://localhost busy (#14).
  console.log = function(...args) {
    originalLog.apply(console, args);
  };
  
  console.info = function(...args) {
    originalInfo.apply(console, args);
  };
  
  console.warn = function(...args) {
    originalWarn.apply(console, args);
    forwardToTauri('warn', args, getSource());
  };
  
  console.error = function(...args) {
    originalError.apply(console, args);
    forwardToTauri('error', args, getSource());
  };
  
  console.debug = function(...args) {
    originalDebug.apply(console, args);
  };
})();
