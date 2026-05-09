# Hyprbar

A custom status bar for Hyprland built with Rust, Wayland, SCTK, wgpu, Vello, and Parley.

The goal of this project is to build a clean, minimal, and highly customizable desktop bar for a Wayland/Hyprland environment.

## Features

- Wayland layer-shell surface.
- GPU-based rendering with wgpu and Vello.
- Text rendering with Parley.
- Centralized theme system.
- Pill-based UI components.
- Date and clock widgets.
- Weather widget using Open-Meteo.
- Hyprland workspace integration through IPC sockets.
- Static command center and notification pills.
- Profile avatar pill loaded from disk.

## Stack

- **Rust**: main language.
- **smithay-client-toolkit**: Wayland client abstractions and layer-shell support.
- **wayland-client**: low-level Wayland protocol interaction.
- **calloop**: event loop integration.
- **wgpu**: graphics backend.
- **vello**: 2D scene rendering.
- **parley**: text shaping and layout.
- **chrono**: date and time.
- **ureq**: weather API requests.
- **serde_json**: JSON parsing.
- **image**: profile avatar decoding and resizing.
- **anyhow**: application-level error handling.
- **env_logger**: logging.

## Project Structure

```txt
src/
├── main.rs                  # Application entry point
├── app.rs                   # Main application coordinator
├── hyprland_ipc.rs          # Hyprland IPC client
├── theme/                   # Centralized design system
│   ├── colors.rs            # Color palette
│   ├── tokens.rs            # Spacing, radius, sizing
│   └── typography.rs        # Font families and text sizes
├── wayland/                 # Wayland / SCTK integration
│   ├── handlers.rs          # SCTK trait implementations
│   ├── init.rs              # Wayland initialization
│   └── layer_surface.rs     # Layer-shell configuration
├── render/                  # Rendering pipeline
│   ├── context.rs           # wgpu + Vello render context
│   ├── geometry.rs          # Basic geometry types
│   └── text.rs              # Parley + Vello text rendering
├── components/              # Reusable UI primitives
│   ├── component.rs         # Component trait
│   ├── mod.rs
│   └── pill.rs              # Base pill background
└── bar/                     # Concrete bar components
    ├── layout.rs            # Left / center / right layout
    ├── arch_logo_pill.rs
    ├── date_pill.rs
    ├── clock_pill.rs
    ├── weather_pill.rs
    ├── command_center_pill.rs
    ├── workspaces_pill.rs
    ├── notifications_pill.rs
    └── profile_pill.rs
```

## Architecture Rules

1. **Theme is the single source of visual truth.**  
   Components should not hardcode colors, sizes, radii, or spacing when those values belong in `Theme`.

2. **Wayland details stay inside the `wayland` module.**  
   The rest of the application should not directly depend on low-level Wayland setup logic.

3. **UI elements implement `Component`.**  
   Components expose:
   - `measure`: calculate intrinsic size.
   - `render`: draw inside the provided bounds.

4. **`Pill::draw` is the visual base for pill components.**  
   Components draw the shared background first, then render their own content on top.

5. **The app state is intentionally centralized.**  
   The Wayland event queue and the calloop loop need a single shared state type, so `AppState` coordinates Wayland, rendering, theme, and UI state.

## Requirements

- Linux.
- A Wayland compositor with `wlr-layer-shell` support.
- Hyprland recommended.
- Rust stable or nightly with edition 2024 support.
- Fonts:
  - `Inter` or a compatible fallback.
  - `Symbols Nerd Font` for icons.

## Running

```sh
cargo run
```

For a release build:

```sh
cargo run --release
```

Enable logs with:

```sh
RUST_LOG=info cargo run
```

Or for more detailed Hyprland event logs:

```sh
RUST_LOG=debug cargo run
```

## Assets

The profile pill currently tries to load:

```txt
assets/profile.jpeg
```

If the image is missing or cannot be decoded, the bar falls back to a simple placeholder circle.

## Hyprland IPC

Hyprbar talks to Hyprland directly through its Unix sockets.

Hyprland exposes sockets under:

```txt
$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/
```

Used sockets:

- `.socket.sock`: request/response queries.
- `.socket2.sock`: event stream.

This is implemented manually instead of using `hyprland-rs`, because older versions of that crate may look for sockets in deprecated locations.

## Current State

Hyprbar currently renders a functional top bar with several pill-based widgets:

- Arch logo.
- Date.
- Clock.
- Weather.
- Command center placeholder.
- Hyprland workspaces.
- Notifications placeholder.
- Profile avatar.

The project is still early-stage, but the current structure is ready to grow component by component.

## Known Limitations

- **No real blur yet.**  
  The pill shadow is currently a simple solid shadow. Real blur would require rendering to an intermediate texture and applying a blur pass.

- **No wallpaper/background blur.**  
  True glassmorphism blur behind the bar is not currently available through standard `wlr-layer-shell`. That would require compositor-level support.

- **Limited input handling.**  
  The current bar focuses on rendering. Clicks, hover states, popovers, and interactive panels still need proper pointer/input handling.

- **Weather location is hardcoded.**  
  The weather widget currently uses a fixed location. This should eventually move to configuration.

- **Notifications are static for now.**  
  The notification pill is present visually, but it does not yet connect to a notification daemon.

## Roadmap Ideas

- Config file support.
- Click handling.
- Hover states.
- Command center popover.
- Notification daemon integration.
- Configurable weather location.
- Workspace animations.
- More robust font fallback.
- Optional Taffy-based layout if the layout becomes more complex.
- CI with `cargo fmt`, `cargo clippy`, and `cargo test`.

## Development Philosophy

Build it in this order:

1. Make it work.
2. Make it clear.
3. Make it beautiful.
4. Then scale the complexity.
