# Plan de pruebas

## Capas

### Unitarias Rust

- Encode y decode de header v1.
- Vector dorado idéntico en RTABC y RTAR.
- Rechazo de magic, versión, longitud y fragmentación inválidos.
- Wrap de secuencia.
- Ring buffers nunca exceden capacidad.
- Jitter buffer, PLC y descarte por deadline.
- Reensamblado HEVC solo entrega frames completos.

### Compatibilidad entre repos

- RTABC genera fixtures que RTAR decodifica.
- RTAR genera fixtures que RTABC decodifica.
- Una prueba falla si cambia cualquier byte no versionado.

### Integración local sin teléfono

- Emisor y receptor corren sobre loopback UDP/QUIC.
- Audio sintético permite verificar orden, duración y clipping.
- Video sintético con patrón y timestamps permite verificar fragmentación y sincronización.
- Proxy de red inyecta pérdida, reorder, duplicación, jitter y límite de ancho de banda.
- Start, stop y reconnect repetidos detectan tareas o sockets huérfanos.

### Plataforma escritorio

- `cargo test --all-targets` en Windows.
- `cargo test --all-targets` en Arch Linux.
- Captura real WASAPI y PipeWire.
- Cambio de dispositivo, suspensión y reanudación.
- Pantalla completa, resize y salida limpia.

### Android

El núcleo Rust puede probarse sin teléfono. Las siguientes condiciones no pueden validarse de forma honesta solo llamando funciones Rust:

- prioridad real del foreground service;
- bloqueo de pantalla y Doze del fabricante;
- consentimiento y finalización de `MediaProjection`;
- encoder HEVC hardware disponible y sus latencias;
- coexistencia física de Wi-Fi 2.4 GHz y Bluetooth.

Estas pruebas requieren emulador para lifecycle básico y al menos un teléfono físico para aceptación.

## Escenarios obligatorios

1. Audio durante 30 minutos con pantalla Android apagada.
2. Pérdida de red de 10 segundos y reconexión automática.
3. Cambio entre Wi-Fi 5 GHz y 2.4 GHz.
4. Audífonos Bluetooth conectados antes y durante sesión.
5. Pantalla 720p60 y 1080p30; rotación vertical y horizontal.
6. Aplicación Android que permite captura de audio y otra que la prohíbe.
7. Bloqueo durante proyección, esperado: Android termina proyección, UI informa y recursos se liberan.
8. Cierre desde bandeja, notificación y botón Stop sin sockets ni mute residuales.

## Métricas

- Latencia de captura a reproducción y captura a presentación.
- p50, p95 y máximo observado.
- pérdida, paquetes tardíos, reorder y jitter.
- underruns de audio, frames de video descartados y solicitudes de keyframe.
- bitrate, CPU, GPU, memoria privada y batería Android.

## Gate por stage

- Formato y lint sin errores nuevos.
- Pruebas unitarias verdes en ambos repos.
- Compatibilidad dorada verde.
- Integración correspondiente verde.
- Warnings nuevos explicados o eliminados.
- Documentación actualizada.
