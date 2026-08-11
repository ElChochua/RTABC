# Decisiones técnicas

## D-001: mantener Rust como núcleo

Rust sigue a cargo de protocolo, transporte, buffers, codecs de audio y lógica de sesión. Reduce duplicación y permite pruebas entre extremos sin UI.

## D-002: mantener `egui` en escritorio

`egui` es suficiente para controles y viewport de video. Cambiar framework no mejora el camino crítico y aumentaría memoria y alcance.

## D-003: Tauri solo como UI Android

Tauri permanece para HTML/CSS. Lifecycle, foreground services, `MediaProjection`, `AudioRecord` y `MediaCodec` requieren implementación nativa Android. Los frames no cruzan JavaScript.

## D-004: HEVC hardware, no x265 software en Android

x265 es un encoder HEVC software, no un decoder. Su licencia es GPLv2 o comercial y su carga CPU no coincide con baja latencia y batería móvil. Android usará `MediaCodec` con HEVC hardware. x265 puede servir como herramienta de fixture o fallback de escritorio solo después de revisión legal.

## D-005: Opus reemplaza PCM de red

PCM es simple pero consume ancho de banda y no oculta pérdida. Opus ofrece frames cortos, PLC y FEC. PCM queda solo como transición y diagnóstico.

## D-006: PipeWire para captura Linux

CPAL no ofrece por sí solo una abstracción portable equivalente a WASAPI loopback. PipeWire permite capturar el sink monitor y está disponible más allá de Arch.

## D-007: QUIC DATAGRAM como transporte objetivo

Medios necesitan entrega no confiable y control necesita entrega confiable. QUIC permite ambos con autenticación, cifrado y control de congestión compartidos. Stage 1 conserva UDP mientras estabiliza framing.

## D-008: 1200 bytes por datagrama

El límite evita depender de fragmentación IP. Video se fragmenta en aplicación y descarta unidades incompletas por deadline.

## D-009: `ruopus` para Stage 2

Se fija `ruopus` 0.1.2 sin features por defecto. Es una implementación Rust sin FFI, mantiene PLC/FEC disponible para pruebas de escritorio y reduce fricción al compilar Android. Antes de producción se medirán compatibilidad, calidad y costo CPU contra libopus nativo.

## Referencias primarias

- [Android MediaProjection](https://developer.android.com/media/grow/media-projection)
- [Android capture de video y audio](https://developer.android.com/media/platform/av-capture)
- [Android background playback](https://developer.android.com/media/media3/session/background-playback)
- [Android MediaCodec](https://developer.android.com/reference/android/media/MediaCodec)
- [PipeWire](https://docs.pipewire.org/)
- [PipeWire capture sink](https://docs.pipewire.org/group__pw__keys.html)
- [Opus RFC 6716](https://www.rfc-editor.org/rfc/rfc6716.html)
- [QUIC DATAGRAM RFC 9221](https://www.rfc-editor.org/rfc/rfc9221.html)
- [RTP timing concepts RFC 3550](https://www.rfc-editor.org/rfc/rfc3550.html)
- [x265 introduction and license](https://x265.readthedocs.io/en/stable/introduction.html)
