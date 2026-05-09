//! `RenderContext` encapsula wgpu (device/queue/surface) + vello (renderer).
//!
//! Vello 0.5+ no expone `render_to_surface`. El flujo es:
//! 1. Renderizar la scene a una textura intermedia (Rgba8Unorm con STORAGE_BINDING).
//! 2. Blittear esa textura al surface usando `wgpu::util::TextureBlitter`.

use anyhow::{anyhow, Context, Result};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use vello::{
    util::{RenderContext as VelloUtilContext, RenderSurface},
    wgpu, AaConfig, Renderer, RendererOptions, Scene,
};

pub struct RenderContext {
    vello_ctx: VelloUtilContext,
    surface: Option<RenderSurface<'static>>,
    renderer: Option<Renderer>,
    intermediate: Option<wgpu::Texture>,
    blitter: Option<wgpu::util::TextureBlitter>,
    pub scene: Scene,
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            vello_ctx: VelloUtilContext::new(),
            surface: None,
            renderer: None,
            intermediate: None,
            blitter: None,
            scene: Scene::new(),
        }
    }

    pub fn create_surface<H>(&mut self, handle: H, width: u32, height: u32) -> Result<()>
    where
        H: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static,
    {
        let mut surface = pollster::block_on(self.vello_ctx.create_surface(
            handle,
            width,
            height,
            wgpu::PresentMode::AutoVsync,
        ))
        .map_err(|e| anyhow!("vello create_surface failed: {e:?}"))?;

        // === Configurar alpha_mode para transparencia ===
        // Hyprland respeta surfaces transparentes solo si el alpha_mode es
        // PreMultiplied o PostMultiplied. Auto/Opaque rellena con negro.
        let device_handle = &self.vello_ctx.devices[surface.dev_id];
        let caps = surface.surface.get_capabilities(&device_handle.adapter());

        let preferred_alpha = [
            wgpu::CompositeAlphaMode::PreMultiplied,
            wgpu::CompositeAlphaMode::PostMultiplied,
            wgpu::CompositeAlphaMode::Inherit,
        ];
        if let Some(&alpha_mode) = preferred_alpha
            .iter()
            .find(|m| caps.alpha_modes.contains(m))
        {
            log::info!("alpha_mode: {:?}", alpha_mode);
            surface.config.alpha_mode = alpha_mode;
            surface.surface.configure(&device_handle.device, &surface.config);
        } else {
            log::warn!(
                "compositor no soporta alpha_modes transparentes. Disponibles: {:?}",
                caps.alpha_modes
            );
        }

        if self.renderer.is_none() {
            let renderer = Renderer::new(&device_handle.device, RendererOptions::default())
                .map_err(|e| anyhow!("Renderer::new failed: {e:?}"))?;
            self.renderer = Some(renderer);
        }

        self.surface = Some(surface);
        self.rebuild_intermediate(width, height);
        Ok(())
    }

    fn rebuild_intermediate(&mut self, width: u32, height: u32) {
        let surface = self.surface.as_ref().expect("surface no inicializado");
        let device = &self.vello_ctx.devices[surface.dev_id].device;
        let surface_format = surface.format;

        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("vello-intermediate"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let blitter = wgpu::util::TextureBlitter::new(device, surface_format);

        self.intermediate = Some(tex);
        self.blitter = Some(blitter);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(s) = &mut self.surface {
            self.vello_ctx.resize_surface(s, width, height);
            self.rebuild_intermediate(width, height);
        }
    }

    pub fn size(&self) -> Option<(u32, u32)> {
        self.surface
            .as_ref()
            .map(|s| (s.config.width, s.config.height))
    }

    pub fn render(&mut self) -> Result<()> {
        let surface = self.surface.as_ref().context("surface no inicializado")?;
        let renderer = self.renderer.as_mut().context("renderer no inicializado")?;
        let intermediate = self
            .intermediate
            .as_ref()
            .context("intermediate no inicializado")?;
        let blitter = self.blitter.as_ref().context("blitter no inicializado")?;
        let device_handle = &self.vello_ctx.devices[surface.dev_id];

        let frame = surface
            .surface
            .get_current_texture()
            .context("get_current_texture failed")?;

        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let intermediate_view = intermediate.create_view(&wgpu::TextureViewDescriptor::default());

        renderer
            .render_to_texture(
                &device_handle.device,
                &device_handle.queue,
                &self.scene,
                &intermediate_view,
                &vello::RenderParams {
                    base_color: vello::peniko::Color::TRANSPARENT,
                    width: surface.config.width,
                    height: surface.config.height,
                    antialiasing_method: AaConfig::Area,
                },
            )
            .map_err(|e| anyhow!("render_to_texture failed: {e:?}"))?;

        let mut encoder =
            device_handle
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("blit"),
                });
        blitter.copy(
            &device_handle.device,
            &mut encoder,
            &intermediate_view,
            &frame_view,
        );
        device_handle.queue.submit([encoder.finish()]);

        frame.present();
        self.scene.reset();
        Ok(())
    }
}

impl Default for RenderContext {
    fn default() -> Self {
        Self::new()
    }
}