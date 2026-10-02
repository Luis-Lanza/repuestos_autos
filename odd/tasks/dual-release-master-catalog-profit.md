# Publicar dos releases consecutivas

## Objetivo
Preservar el estado actual de `master` como una release inmutable y publicar la integración de `feat/catalog-access-profit-report` como la release siguiente.

## Decisiones
- La base actual `master` (`f1d9950`) se publica primero como `v1.0.0`.
- La rama de funcionalidades se integra después y el nuevo `master` se publica como `v1.1.0`.
- No se incorporan notas ODD ni archivos no rastreados ajenos a esta rama.
- Cada release se identifica con un tag anotado, publicado en `origin`, y su GitHub Release correspondiente.

## Tareas
- [x] R1 Congelar y verificar el candidato actual de `master`; publicar `v1.0.0`. Evidencia: tag anotado en `f1d995073059daf3ce3b72f348f07544f5ca6909`, enviado a `origin`; GitHub Release publicada en `https://github.com/Luis-Lanza/repuestos_autos/releases/tag/v1.0.0`.
- [x] R2 Aislar cambios locales ajenos, verificar el candidato de la rama e integrar solamente sus commits en `master`. Evidencia: integración no-fast-forward `1658c0e` (`feat: release catalog access and profit reports`) enviada a `origin/master`; los seis archivos no rastreados ajenos permanecieron sin incluirse.
- [ ] R3 Verificar el `master` integrado; publicar `v1.1.0` y registrar la evidencia.

## Criterios de aceptación
- `v1.0.0` apunta exactamente al `master` previo a la integración.
- `v1.1.0` apunta exactamente al `master` posterior a integrar la rama.
- Ambos tags y releases son visibles en el remoto.
- Ningún archivo local ajeno entra en la integración.
