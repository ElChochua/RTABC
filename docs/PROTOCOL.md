# LocalAudioLink Protocol v1

## Estado

Stage 2 usa el envelope binario y Opus en RTABC y RTAR. Discovery y control todavía usan mensajes heredados. HEVC, QUIC y pairing llegan en etapas posteriores sin cambiar el significado del header v1.

## Orden de bytes

Todos los enteros multibyte del header usan big-endian.

## Header fijo

Longitud: 32 bytes.

| Offset | Tamaño | Campo | Descripción |
| --- | ---: | --- | --- |
| 0 | 4 | magic | ASCII `LAL1` |
| 4 | 1 | version | `1` |
| 5 | 1 | media_kind | audio=1, video=2, control=3, heartbeat=4 |
| 6 | 1 | codec | none=0, pcm_s16le=1, opus=2, hevc=3, json=4 |
| 7 | 1 | flags | bit 0 keyframe, bit 1 config, bit 2 discontinuity, bit 3 end |
| 8 | 4 | stream_id | Identificador dentro de sesión |
| 12 | 4 | sequence | Incrementa por datagrama, con wrap |
| 16 | 8 | timestamp_us | Tiempo monotónico de captura |
| 24 | 2 | payload_len | Bytes después del header |
| 26 | 2 | fragment_index | Índice desde 0 |
| 28 | 2 | fragment_count | Total, mínimo 1 |
| 30 | 2 | reserved | Debe emitirse como cero |

## Límites

- Datagrama completo máximo: 1200 bytes.
- Payload máximo v1: 1168 bytes.
- `fragment_count` no puede ser cero.
- `fragment_index` debe ser menor que `fragment_count`.
- Longitud declarada debe coincidir exactamente con datagrama recibido.
- Magic, versión, tipo o codec desconocidos se rechazan.

## Secuencia y timestamps

- Cada stream mantiene su secuencia independiente.
- Un salto indica pérdida; un valor anterior indica reordenamiento o duplicado.
- Todos los fragmentos del mismo frame de video comparten timestamp.
- Audio incrementa timestamp por instante de captura, no por instante de envío.
- El receptor no espera indefinidamente un paquete ausente.

## Audio

- Stage 2: Opus, 48 kHz, estéreo, frames de 10 ms y bitrate objetivo de 128 kbit/s.
- Cada datagrama contiene exactamente un paquete Opus; no se fragmenta.
- El receptor precarga 2 paquetes (20 ms) y conserva como máximo 8 (80 ms).
- Paquetes duplicados o tardíos se descartan. El emisor descarta audio viejo si la cola supera 30 ms.
- Ante pérdida se intenta FEC con el paquete siguiente; si no está disponible o no aplica al modo Opus activo, se usa PLC.
- Más de 6 pérdidas consecutivas sin datos reinicia el jitter buffer y el decoder.
- `discontinuity` reinicia encoder, decoder y ordenamiento.

## Video HEVC

- Payload transporta Annex B NAL units.
- VPS, SPS y PPS usan flag `config` y se repiten con cada keyframe.
- Primer fragmento de IDR usa `keyframe`.
- Sin B-frames. El encoder prioriza baja latencia sobre compresión máxima.
- Si falta un fragmento, se descarta la unidad completa. No se entrega un frame corrupto al decoder.

## Control objetivo

El canal confiable negocia versión, roles, codecs, resolución, framerate, bitrate, audio, MTU y capabilities. Mensajes mínimos: `hello`, `offer`, `answer`, `start`, `stop`, `stats`, `request_keyframe`, `set_bitrate`, `ping`, `pong` y `error`.

## Compatibilidad

El vector dorado vive en:

- `RTABC/src/protocol.rs`
- `RTAR/src-tauri/src/protocol.rs`

Ambas pruebas deben producir exactamente los mismos 34 bytes. Un cambio al vector requiere actualización deliberada de versión y documentación.
