# arch-bar

Status bar para Hyprland (Wayland) construida sobre SCTK + wgpu + vello.

## Stack

- **smithay-client-toolkit**: cliente Wayland + layer-shell.
- **wgpu**: backend gráfico.
- **vello**: 2D scene API sobre wgpu.
- **parley**: shaping y layout de texto.
- **chrono**: fecha/hora.

## Estructura

```
src/
├── main.rs              # Entry point
├── app.rs               # Coordinator: Wayland ↔ render loop
├── theme/               # Sistema de diseño centralizado
│   ├── colors.rs        # Paleta
│   ├── tokens.rs        # Spacing, radio, sizing
│   └── typography.rs    # Fuente, tamaños
├── wayland/             # Abstracción sobre SCTK
│   ├── connection.rs    # Estado Wayland + dispatch traits
│   └── layer_surface.rs # Config de layer-shell (anchor, capa, etc.)
├── render/              # Pipeline gráfico
│   ├── context.rs       # wgpu + vello renderer
│   └── text.rs          # parley + vello text rendering
├── components/          # Primitives reutilizables
│   ├── component.rs     # Trait Component
│   └── pill.rs          # Primitive visual: pill con sombra
└── bar/                 # La barra concreta
    ├── layout.rs        # Layout 3 secciones (left/center/right)
    └── date_pill.rs     # Pill de fecha (dd/mm)
```

## Reglas de arquitectura

1. **El theme es la única fuente de estilos.** Ningún componente hardcodea colores, tamaños ni radios. Todo sale de `Theme`.
2. **El módulo wayland es opaco hacia adentro.** El resto del código no toca `wayland-client` ni `sctk` directamente.
3. **Los componentes implementan `Component`.** Trait con `measure` (intrinsic size) y `render` (dibujar en bounds).
4. **`Pill::draw` es el chasis visual de todo.** Cualquier componente que quiera el look glassy lo llama primero, después dibuja su contenido.

## Cómo crece

- **Nuevo componente** → archivo en `bar/`, implementar `Component`, agregar al `Bar::new()`.
- **Cambiar look** → editar `theme/`. Todos los componentes se actualizan.
- **Más interacción (click, hover)** → agregar input handling en `wayland/` y métodos al trait `Component`.
- **Animaciones** → reemplazar `blocking_dispatch` por un timer de 16ms en `app.rs` y agregar estado animable en componentes.
- **Cuando haya 5+ componentes** → migrar `bar/layout.rs` a `taffy` para flexbox real.

## Build

```sh
cargo run --release
```

Requiere un compositor con soporte de `wlr-layer-shell` (Hyprland, Sway, river, etc.).

## Estado actual

Skeleton funcional. Renderiza una pill con la fecha en formato dd/mm en la esquina superior izquierda. Listo para crecer.

## Limitaciones conocidas

- **Sombra fake.** Vello no tiene blur built-in. Para sombra con blur real, pre-renderizar la pill a una textura y aplicar un compute shader de blur. Para glassmorphism con blur del wallpaper detrás, no es posible con `wlr-layer-shell` actualmente — el compositor tendría que exponer una extensión.
- **Re-render solo en eventos.** El loop bloquea con `blocking_dispatch`. Para refrescar la fecha cada minuto o animar, necesitás un timer paralelo (usar `calloop` o `tokio` + integración con el event queue).
- **Sin input.** El layer está en `KeyboardInteractivity::None` y no hay handling de mouse aún. Cuando agregues popovers/click, hay que manejar `wl_pointer`.
