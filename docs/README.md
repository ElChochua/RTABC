# LocalAudioLink: documentación de producto

Este directorio define el contrato del conjunto RTABC + RTAR. Ante una diferencia entre el README histórico y estos documentos, estos documentos tienen prioridad.

## Documentos

- [PRODUCT_SCOPE.md](PRODUCT_SCOPE.md): alcance, modos y criterios de aceptación.
- [ARCHITECTURE.md](ARCHITECTURE.md): componentes, plataformas y flujos de datos.
- [PROTOCOL.md](PROTOCOL.md): contrato binario y negociación de medios.
- [UX_DESIGN.md](UX_DESIGN.md): estructura y reglas visuales de ambas aplicaciones.
- [STAGES.md](STAGES.md): orden de implementación y salida de cada etapa.
- [TEST_PLAN.md](TEST_PLAN.md): pruebas automáticas, integración y validación física.
- [DECISIONS.md](DECISIONS.md): decisiones técnicas que no deben reinterpretarse sin un ADR nuevo.

## Regla de cambio

Todo cambio de protocolo, modo de operación, plataforma soportada o criterio de aceptación debe actualizar primero estos documentos. Los cambios incompatibles incrementan la versión mayor del protocolo.
