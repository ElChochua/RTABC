# Alcance de LocalAudioLink

## Objetivo

LocalAudioLink transmite medios en una LAN con la menor latencia estable posible y memoria acotada. No persigue calidad de archivo ni transmisión por Internet.

## Modos obligatorios

### Audio PC a teléfono

RTABC captura la salida de audio del PC. RTAR descubre, conecta, recibe, decodifica y reproduce el audio, incluso con pantalla apagada.

### Pantalla y audio del teléfono a PC

RTAR captura la pantalla y, cuando Android lo permite, el audio reproducido por aplicaciones. RTABC recibe, sincroniza y presenta ambos. El video puede ocupar la ventana completa o pantalla completa.

### Roles

Ambas aplicaciones pueden iniciar o aceptar una sesión según el modo elegido. “Servidor” y “cliente” son responsabilidades de una sesión, no identidades permanentes de una aplicación.

## Plataformas

- Escritorio: Windows 10/11 y Linux con PipeWire. Arch Linux es plataforma de validación, pero el código no dependerá de herramientas exclusivas de Arch.
- Móvil: Android 10 o posterior para permitir captura de audio de reproducción mediante API 29.
- Fuera de alcance inicial: macOS, iOS, retransmisión por Internet y múltiples receptores simultáneos.

## Medios

- Audio de red final: Opus, 48 kHz, estéreo, frames cortos, PLC y FEC configurables.
- Compatibilidad temporal de Stage 1: PCM S16LE.
- Video: H.265/HEVC, 8 bits, máximo 1920x1080, sin B-frames.
- Android codifica HEVC mediante `MediaCodec` hardware. x265 software no será el camino principal por consumo, latencia y licencia GPL/comercial.
- El receptor conserva relación de aspecto. No escala por encima de 1080p.

## Comportamiento de red

- 5 GHz o 6 GHz es la ruta recomendada cuando se usan audífonos Bluetooth.
- 2.4 GHz con Bluetooth debe degradar bitrate o resolución antes de acumular latencia.
- Pérdida de paquetes nunca debe reproducir memoria sin inicializar, ruido digital fuerte ni bloquear la UI.
- Audio perdido usa PLC o silencio corto. Video perdido descarta hasta punto recuperable y solicita keyframe.
- La cola de reproducción tiene límite estricto. Al superar el límite se descarta contenido antiguo.

## Objetivos de aceptación

Estos valores son objetivos, no resultados ya medidos.

| Métrica | Objetivo inicial |
| --- | --- |
| Audio PC a Android, Wi-Fi 5 sin Bluetooth | p50 hasta 50 ms, p95 hasta 100 ms |
| Video Android a PC, 720p60 o 1080p30 | p50 hasta 120 ms, p95 hasta 200 ms |
| Resolución máxima | 1920x1080 |
| Datagramas de medios | máximo 1200 bytes |
| Memoria RTABC recibiendo 1080p | objetivo menor a 250 MiB |
| Memoria RTAR en audio | objetivo menor a 180 MiB |
| Recuperación de pérdida temporal | sin reinicio manual de aplicación |

Bluetooth A2DP agrega latencia fuera del control de LocalAudioLink. El producto minimiza su propia contribución, pero no promete latencia total “cero”.

## Privacidad y límites Android

- Cada captura de pantalla requiere consentimiento visible de Android.
- Bloquear el teléfono puede terminar `MediaProjection`; RTAR debe notificarlo y liberar recursos.
- Algunas aplicaciones prohíben capturar su audio o superficie. RTAR no intenta evadir DRM ni políticas de captura.
- El audio de fondo recibido se ejecuta mediante foreground service y notificación persistente mientras la sesión está activa.

## Terminado significa

Una feature está terminada solo cuando existe en ambos extremos necesarios, tiene prueba automática del núcleo, informa errores en UI y pasa los criterios aplicables de [TEST_PLAN.md](TEST_PLAN.md).
