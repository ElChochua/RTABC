# Stages de implementación

Cada stage termina con código funcional en los extremos afectados, pruebas y actualización documental. No se inicia el siguiente con pruebas rojas.

## Baseline y contrato

Estado: completado.

Salida:

- Auditoría de RTABC y RTAR.
- Alcance, arquitectura, protocolo, UX y plan de pruebas.
- Envelope binario v1 integrado en audio actual.
- Vector dorado idéntico en ambos repos.

## Audio resistente

Estado: completado. Implementación actual fija frames de 10 ms; la negociación de 5 ms queda para la etapa de transporte adaptativo.

Salida:

- Opus 48 kHz estéreo con frames de 10 ms.
- Secuencia, métricas de pérdida, PLC y jitter buffer acotado.
- Simulador de pérdida, duplicación, reorder y jitter.
- Perfiles de calidad que no acumulan audio viejo.

## Lifecycle Android

Salida:

- Foreground service real para recepción de audio.
- `MediaSession` y notificación.
- Eliminación de wake lock de pantalla como mecanismo principal.
- Propiedad segura de streams sin punteros globales manuales.
- Pruebas de start, stop, reconnect y cancelación del núcleo.

## Captura Linux

Salida:

- Trait común de captura.
- WASAPI aislado para Windows.
- PipeWire sink monitor para Linux.
- Mute implementado solo donde sea seguro; UI informa capability.
- Compilación y smoke test en Windows y Arch Linux.

## Roles y UI

Salida:

- State machine común de sesión.
- Rediseño `egui` y HTML/CSS según [UX_DESIGN.md](UX_DESIGN.md).
- Discovery y pairing sin lenguaje de servidor público.
- Estados de carga, vacío, reconexión y error.

## Pantalla Android a PC

Salida:

- Foreground service `mediaProjection`.
- Captura de pantalla a `MediaCodec` HEVC hardware.
- Captura de audio permitida a Opus.
- Fragmentación v1, reensamblado acotado y solicitud de keyframe.
- Decode HEVC, sincronización A/V, render `egui` y pantalla completa.

## Transporte seguro y adaptación

Salida:

- QUIC DATAGRAM para medios y stream confiable para control.
- Pairing y autenticación local.
- Estimación de congestión, bitrate y resolución adaptativos.
- Perfiles probados en 5 GHz y 2.4 GHz con Bluetooth.

## Empaquetado y aceptación

Salida:

- Instalador Windows.
- Paquetes Linux documentados, al menos AppImage o tarball y receta Arch.
- APK Android firmado para pruebas.
- Matriz completa de [TEST_PLAN.md](TEST_PLAN.md).
- Métricas reales comparadas con objetivos de producto.
