# Process 26290 Analysis - WebKit GPU Process

## Process Identity
- **PID**: 26290
- **Name**: `com.apple.WebKit.GPU`
- **Type**: WebKit GPU XPC Service
- **Purpose**: Handles GPU-accelerated rendering for the Tauri webview

## Current Status
- **CPU Usage**: 0.0% (mostly idle)
- **Memory**: 72MB
- **Threads**: 5 threads
  - Main thread (event loop)
  - JavaScriptCore scavenger
  - 2x caulk messenger threads
  - 2x RemoteRenderingBackend work queues

## What It Does During Spikes

### Normal Operation
- Mostly idle, waiting in event loop (`mach_msg` calls)
- Processes GPU rendering requests from the webview
- Handles layer tree updates and compositing

### During CPU Spikes
The GPU process spikes when:
1. **Layer tree updates** - When the webview content changes
2. **Animation rendering** - When SVG ring gauges animate
3. **Display refresh** - When `requestAnimationFrame` triggers redraws
4. **Compositing** - When WebKit composites layers for display

## Relationship to Graphics/Media CPU

The "Graphics and Media" CPU usage in Activity Monitor includes:
- **Backend process** (main process) - WebKit coordination
- **GPU process** (26290) - Actual GPU rendering
- **WebContent process** - JavaScript execution and DOM

When animations run, all three processes work together:
1. Backend: Coordinates updates
2. WebContent: Executes JavaScript animations
3. GPU: Renders the final frames

## Optimization Impact

The animation optimizations we applied should reduce GPU process spikes:
- **20fps instead of 30fps** = 33% fewer rendering requests
- **Optimized requestAnimationFrame** = Fewer unnecessary redraws
- **Result**: GPU process should spike less frequently

## Monitoring

To see GPU spikes in real-time:
```bash
# Monitor GPU process
watch -n 1 'ps -p 26290 -o pid,pcpu,pmem,comm'

# Or sample during spike
sudo sample 26290 10 -f gpu_sample.txt
```

## Conclusion

Process 26290 is **WebKit's GPU process** - it's normal and expected. The spikes occur during:
- Animation rendering (ring gauges)
- Layer tree updates
- Display refresh cycles

The optimizations we applied should reduce these spikes by reducing animation frequency.
