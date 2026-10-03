# Prueba de modelos en el iPhone (fase A)

**Resumen:** Qwen3.5-2B se queda como modelo por defecto. LFM2.5-1.2B queda como opción ligera y rápida.

Medido el 1 de octubre de 2026 en las siguientes condiciones:

- **Equipo:** iPhone 12 Pro (A14, 6 GB, iOS 26.6.1).
- **Motor:** llama.cpp b11321 con Metal, contexto de 4096 tokens y 2 hilos.
- **Memoria:** con el entitlement `increased-memory-limit`.
- **Conexión:** el iPhone estuvo conectado por cable y cargando durante toda la prueba.

Los modelos se midieron uno tras otro: primero Qwen3.5-2B y después LFM2.5-1.2B, que empezó con el iPhone ya caliente. Qwen2.5-1.5B no se midió porque no estaba en el iPhone.

Los datos completos están en [llm-bench-iphone12pro.json](llm-bench-iphone12pro.json). «GB» son 2³⁰ bytes, como en el monitor de la app.

## Resultados

| | Qwen3.5-2B Q4_K_M | LFM2.5-1.2B Q4_K_M |
|---|---|---|
| Archivo | 1.19 GB | 0.68 GB |
| Parámetros | 1.9 B | 1.2 B |
| Carga en frío / en caliente | 2.0 s / 0.48 s | 1.1 s / 0.15 s |
| Primera palabra en una conversación nueva | 1.3–2.0 s | 0.8–1.4 s |
| Procesar prompt (pp512) | 162 tok/s | 214 tok/s ¹ |
| Generar (tg128) | 18.5 tok/s | 32.3 tok/s ¹ |
| Generando sin parar 3 min | 13 tok/s al empezar, 8.6 tok/s desde el minuto 1 ² | 9.1 tok/s ² |
| Memoria de la app (pico) | 0.20 GB | 0.17 GB |
| Margen de memoria mínimo | 3.80 GB | 3.83 GB |
| 3 ciclos de carga y descarga | sin fallos | sin fallos |
| Calidad (10 preguntas, de 1 a 5) | 3.7 | 3.3 |

¹ Medido con el iPhone ya caliente; en frío debería ser algo más.
² Limitado por la pausa térmica de la app (ver «Calor»).

### Memoria

Los pesos se leen del archivo con mmap y no cuentan en la memoria de la app. Con cualquiera de los dos modelos la app no pasó de 0.2 GB y el margen se quedó por encima de 3.8 GB. Sin el entitlement, el margen era de unos 3 GB.

### Calor

Generando sin parar con el cable conectado, el iPhone pasa a «serio» en menos de un minuto. En ese estado el motor hace una pausa de 100 ms por token. Como la GPU calcula el siguiente token durante la pausa, la velocidad se queda en unos 9 tok/s con cualquier modelo. Las cifras de la prueba sostenida miden esa pausa, no el límite del chip. En 3 minutos nunca llegó a «crítico».

### Calidad

Una respuesta por pregunta, con temperatura 0.7, puntuada a mano:

| # | Pregunta | Qwen3.5-2B | LFM2.5-1.2B |
|---|---|---|---|
| 1 | ¿Cómo van mi batería y mi memoria? | 5 | 2: mezcla cifras («3.9 GB disponibles en un total de 0.1 GB») |
| 2 | La RAM en dos frases | 4: una sola frase | 5: usa los datos del iPhone |
| 3 | 3 consejos para que no se caliente | 4 | 3: «mantén la batería al 50 %» |
| 4 | Resumir una frase | 4: añade algo que no estaba | 5 |
| 5 | 17 × 23 | 1: 351 | 1: 486 |
| 6 | Haiku | 4 | 4 |
| 7 | Traducir al inglés | 5 | 3: «Your phone» en vez de «My phone» |
| 8 | 45 GB ÷ 1.5 GB | 5 | 5 |
| 9 | 3 pasos para liberar espacio | 2: inventa menús y dice que reiniciar borra la caché | 4: genérico pero correcto |
| 10 | ¿Qué es el estado térmico «serio»? | 3: bien al principio, luego exagera y dice que la app ve el consumo de otras apps | 1: dice que se está enfriando para rendir más |

Los dos fallan la multiplicación: no conviene fiarse de sus cuentas.

## Decisión

**Qwen3.5-2B sigue como modelo por defecto.** Cumple los tres requisitos de la fase A:

- **Velocidad:** 8.6 tok/s sostenidos (mínimo: 8).
- **Memoria:** 0.20 GB de pico (máximo: 2 GB).
- **Estabilidad:** 3 ciclos de carga y descarga sin fallos.

Además, contesta mejor que LFM2.5 sobre el propio iPhone (preguntas 1 y 10) y escribe mejor en español.

**LFM2.5-1.2B queda como opción ligera.** Pesa la mitad, carga en la mitad de tiempo y en frío genera 1.75 veces más rápido. A cambio, se equivoca más con los datos del iPhone.

**Qwen2.5-1.5B** sigue en el catálogo para descargarlo desde la app.

## Límites de la prueba

- La prueba sostenida duró 3 minutos en lugar de 10.
- La batería no se pudo medir: el iPhone estaba conectado y se quedó en 100 %.
- LFM2.5 no se midió con el iPhone frío.
- Hay una sola respuesta por pregunta y la puntuación es manual.

## Cambios que salieron de la prueba

- **Datos del iPhone en un mensaje de sistema aparte.** Cuando iban dentro de la pregunta, los modelos los copiaban en la respuesta.
- **Prompt en bloques de 64 tokens.** Con bloques de 512, «Detener» tardaba segundos.
- **Cada turno continúa exactamente el prompt del anterior.** Qwen3.5 y LFM2.5 son modelos híbridos: solo reutilizan lo ya procesado si el prompt nuevo empieza igual que el anterior. Para conseguirlo:
  - el historial incluye los datos del iPhone que se dieron con cada pregunta;
  - las respuestas se guardan sin recortar;
  - el bloque `<think>` vacío se repite en las respuestas anteriores;
  - el historial se recorta a saltos de medio presupuesto, no en cada turno.

## Cómo repetirla

Con el iPhone desbloqueado y la app en primer plano (la app mantiene la pantalla encendida):

```bash
xcrun devicectl device process launch --device <id> --terminate-existing \
  --environment-variables '{"IOS_STATS_BENCH":"all","IOS_STATS_SOAK_SECS":"180"}' com.gilberto.iosstats
```

- **Avance:** aparece en un aviso en pantalla y en `Documents/llm-bench-progress.jsonl`.
- **Final:** al terminar se crea `Documents/llm-bench.done`.

Para traer los resultados al Mac:

```bash
xcrun devicectl device copy from --device <id> --domain-type appDataContainer \
  --domain-identifier com.gilberto.iosstats --source Documents/llm-bench.json --destination llm-bench.json
```
