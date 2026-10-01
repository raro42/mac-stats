# iOS Stats

App de iPhone derivada de [mac-stats](../README.md), con dos partes:

- **Monitor:** CPU, RAM, memoria de la app, espacio, batería, estado térmico, modo de bajo consumo, red e historial de 5 min / 1 h.
- **Chat con IA local:** un modelo de lenguaje pequeño que corre **dentro del iPhone** con [llama.cpp](https://github.com/ggml-org/llama.cpp), sin servidor ni conexión. Ollama no funciona en iOS; llama.cpp es el motor en el que se basa y usa los mismos modelos GGUF.

Es un proyecto Tauri 2 independiente dentro del repo: no toca la app de Mac.

## Estructura

| Ruta | Qué hay |
|---|---|
| `src/` | Interfaz (Vite + TypeScript, sin framework): monitor, chat y laboratorio |
| `src-tauri/src/metrics/` | Lecturas del iPhone en Rust (Mach, Foundation, UIKit, `getifaddrs`) |
| `src-tauri/src/chat/` | Catálogo de modelos, prompt, conversaciones y comandos del chat |
| `src-tauri/src/lab.rs`, `selftest.rs` | Laboratorio, medición automática y autodiagnóstico (depuración) |
| `plugins/tauri-plugin-llm/` | Plugin de Tauri: `LlamaEngine.swift` (llama.cpp), `ModelStore.swift` (descargas) |
| `models/catalog.json` | Modelos disponibles, fijados a un commit de Hugging Face con su SHA-256 |
| `scripts/` | `build-llama.sh` (compila llama.cpp) y `fetch-models.sh` (descarga modelos en el Mac) |
| `vendor/` | `llama.xcframework` compilado y su manifiesto de privacidad (el framework no va a git) |

## Requisitos

- macOS con Xcode 26 y una cuenta de desarrollador iniciada en Xcode (*Ajustes → Cuentas*).
- Rust con los targets de iOS: `rustup target add aarch64-apple-ios aarch64-apple-ios-sim`.
- `brew install xcodegen cmake` y pnpm.

## Preparación

```bash
pnpm install
./scripts/build-llama.sh          # llama.cpp b11321 para iPhone y simulador (~15 min la primera vez)
pnpm tauri ios init               # solo si falta src-tauri/gen/apple
```

`tauri ios init` reescribe el archivo de entitlements; el `build.rs` del plugin vuelve a añadir `increased-memory-limit` en cada build.

## Ejecutar

```bash
# Simulador (el modelo corre en CPU)
pnpm tauri ios build --debug --target aarch64-sim
xcrun simctl install booted "src-tauri/gen/apple/build/arm64-sim/iOS Stats.app"
xcrun simctl launch booted com.gilberto.iosstats

# iPhone conectado por cable (GPU con Metal)
pnpm tauri ios build --debug --export-method debugging
xcrun devicectl device install app --device <id> "src-tauri/gen/apple/build/arm64/iOS Stats.ipa"
xcrun devicectl device process launch --device <id> --terminate-existing com.gilberto.iosstats
```

Compila siempre desde la Terminal: las builds lanzadas desde la interfaz de Xcode no encuentran Node ni pnpm.

## Modelos

La forma normal es descargarlos desde la app (pestaña Chat): solo por Wi‑Fi por defecto, con reanudación y verificación SHA-256. Para pruebas también se pueden bajar en el Mac (`./scripts/fetch-models.sh`) y copiarlos al iPhone, aunque `devicectl` copia a ~1 MB/s:

```bash
xcrun devicectl device copy to --device <id> --domain-type appDataContainer \
  --domain-identifier com.gilberto.iosstats --source models/<archivo>.gguf \
  --destination "Library/Application Support/models/<archivo>.gguf"
```

Los resultados de la prueba de modelos están en [docs/llm-spike.md](docs/llm-spike.md).

## Herramientas de depuración

Solo existen en builds de depuración. Se pasan como variables de entorno al lanzar la app: en el iPhone con `devicectl ... process launch --environment-variables '{"VAR":"valor"}'` y en el simulador con el prefijo `SIMCTL_CHILD_`.

| Variable | Qué hace |
|---|---|
| `IOS_STATS_BENCH=all` o `<id>,<id>` | Mide los modelos (carga, velocidad, memoria, temperatura, 10 preguntas en es-MX) y guarda `Documents/llm-bench.json` |
| `IOS_STATS_SOAK_SECS`, `IOS_STATS_THREADS` | Duración de la prueba sostenida (600 s) e hilos (2) para la medición |
| `IOS_STATS_SELFTEST=1` | Autodiagnóstico del chat; guarda `Documents/selftest.json` |
| `IOS_STATS_FAKE_THERMAL=serious\|critical` | Simula el estado térmico en el monitor y en el motor |
| `IOS_STATS_DEMO_PROMPT="…"` | Abre el chat y envía esa pregunta al arrancar |

La pestaña Chat incluye además una tarjeta «Laboratorio» para cargar, medir y probar modelos a mano.

## Pruebas

```bash
(cd src-tauri && cargo test)   # métricas, historial, red, catálogo, prompt y conversaciones
pnpm build                     # tipos de TypeScript y empaquetado
```

## Límites de iOS

- No hay temperaturas en °C, uso de GPU, frecuencia ni lista de procesos: iOS solo expone el estado térmico (4 niveles).
- La app se congela en segundo plano: el monitor deja un hueco en el historial y la generación se cancela.
- El modelo debe caber en el margen de memoria de la app (~3 GB en un iPhone 12 Pro).
