# Diseño de producto

## Dirección

Herramienta técnica minimalista. El contenido principal es la transmisión, no decoración. Variación visual 3/10, movimiento 2/10 y densidad 5/10.

## Sistema visual

- Tema claro y oscuro según sistema, sin mezclar temas dentro de una vista.
- Neutros fríos y un solo acento azul moderado.
- Radio consistente de 8 px para controles y 12 px para paneles.
- Sin gradientes decorativos, glow, glassmorphism ni animaciones perpetuas.
- Números de latencia, pérdida, bitrate y FPS usan fuente monoespaciada.
- Color de estado tiene texto o icono asociado; nunca depende solo del color.

## Escritorio `egui`

### Vista inicial

- Selector de modo: “Enviar audio” o “Recibir pantalla”.
- Dispositivo local y destino descubierto.
- Botón primario único para iniciar.
- Estado de red compacto: banda estimada, latencia, pérdida y codec.

### Receptor de pantalla

- Canvas de video ocupa todo espacio disponible.
- Doble clic o `F11` activa pantalla completa.
- Controles aparecen por interacción y se ocultan durante reproducción.
- `Esc` sale de pantalla completa.
- Panel lateral opcional contiene calidad, audio, estadísticas y desconexión.
- Estados explícitos: sin dispositivo, conectando, esperando permiso, recibiendo, recuperando, error.

### Emisor de audio

- Selección de fuente.
- Opción “Silenciar salida local” solo cuando backend la soporta.
- Perfil “Baja latencia”, “Equilibrado” y “Red congestionada”.
- No mostrar “Public Server”; la aplicación opera solo en LAN.

## Android Tauri

### Navegación

Dos acciones principales: “Escuchar PC” y “Compartir pantalla”. No se usan menús profundos.

### Escuchar PC

- Dispositivo encontrado, estado de conexión y salida de audio.
- Inicio y finalización explícitos.
- Aviso visible de servicio en segundo plano.
- Controles multimedia integrados con notificación y `MediaSession`.

### Compartir pantalla

- Selección del PC receptor.
- Calidad automática por defecto, con límite 720p o 1080p.
- Permiso de captura solicitado justo al iniciar.
- Estado persistente y acción “Detener” tanto en app como notificación.
- Rotación cambia resolución negociada sin reiniciar aplicación completa.

## Rendimiento UI

- Estadísticas se actualizan máximo cuatro veces por segundo.
- Ningún frame de audio o video atraviesa JavaScript.
- Render de video no provoca relayout de paneles.
- Animaciones se limitan a feedback de botón y transición de estado.
