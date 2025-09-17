pub mod circle;

mod mesh;
mod render;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    monitor::MonitorHandle,
    window::{Fullscreen, Window, WindowAttributes, WindowId},
};

use crate::{circle::Circle, render::Renderer};

pub struct Bounds {
    pub width: f32,
    pub height: f32,
}

impl Bounds {
    pub fn half_extents(&self) -> (f32, f32) {
        (self.width / 2.0, self.height / 2.0)
    }
}

pub struct SimulationConfig {
    pub fullscreen: bool,
    pub fps: u64,
}

pub trait Simulation {
    type Instance: Into<Circle> + Copy;

    fn init(&mut self, bounds: Bounds);
    fn step(&mut self, dt: f32, bounds: Bounds);
    fn instances(&self) -> &[Self::Instance];
}

pub fn run_with<S: Simulation + 'static>(sim: S, config: SimulationConfig) -> anyhow::Result<()> {
    pub struct App<S: Simulation> {
        window: Option<Arc<Window>>,
        renderer: Option<Renderer>,
        simulation: S,
        last_frame: Instant,
        config: SimulationConfig,
    }

    impl<S: Simulation> ApplicationHandler for App<S> {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            let monitors: Vec<MonitorHandle> = event_loop.available_monitors().collect();

            if let Ok(window) = event_loop.create_window(
                WindowAttributes::default()
                    .with_title("Particle Simulation")
                    .with_inner_size(
                        if self.config.fullscreen
                            && let Some(monitor) = monitors.first()
                        {
                            let size = monitor.size();
                            LogicalSize::new(size.width as f64 * 0.9, size.height as f64 * 0.8)
                        } else {
                            LogicalSize::new(800.0, 600.0)
                        },
                    ),
            ) {
                if self.config.fullscreen {
                    window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                }

                let window = Arc::new(window);
                let size = window.inner_size();
                let Ok(mut renderer) =
                    pollster::block_on(async { Renderer::new(window.clone(), size).await })
                else {
                    log::error!("Failed to create renderer");
                    event_loop.exit();
                    return;
                };

                self.simulation.init(Bounds {
                    width: size.width as f32,
                    height: size.height as f32,
                });

                renderer.upload_instances(self.simulation.instances());

                self.window = Some(window.clone());
                self.renderer = Some(renderer);
                self.last_frame = Instant::now();
            }
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            _window_id: WindowId,
            event: WindowEvent,
        ) {
            let (Some(window), Some(renderer)) = (self.window.as_ref(), self.renderer.as_mut())
            else {
                return;
            };

            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(size) => renderer.resize(size),
                WindowEvent::ScaleFactorChanged {
                    mut inner_size_writer,
                    ..
                } => {
                    let new_size = window.inner_size();

                    if let Err(e) = inner_size_writer.request_inner_size(new_size) {
                        log::error!("Failed to change inner size: {e}");
                    }

                    renderer.resize(new_size);
                }
                WindowEvent::RedrawRequested => {
                    log::info!("FPS: {}", 1.0 / (self.last_frame.elapsed().as_secs_f32()));

                    let PhysicalSize { width, height } = window.inner_size();

                    if width == 0 || height == 0 {
                        return;
                    }

                    let now = Instant::now();
                    let dt = (now - self.last_frame).as_secs_f32();
                    let bounds = Bounds {
                        width: width as f32,
                        height: height as f32,
                    };

                    self.last_frame = now;

                    if window.has_focus() {
                        self.simulation.step(dt, bounds);
                    }

                    renderer.upload_instances(self.simulation.instances());

                    if let Err(err) = renderer.render() {
                        use wgpu::SurfaceError::*;

                        match err {
                            Timeout => (),
                            Lost | Outdated => {
                                renderer.resize(window.inner_size());
                            }
                            OutOfMemory | Other => {
                                log::error!("wgpu exiting: {err}");
                                event_loop.exit();
                            }
                        }
                    }
                }

                _ => {}
            }
        }

        fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
            let now = Instant::now();

            if now - self.last_frame >= Duration::from_millis(1000 / self.config.fps) {
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            } else {
                event_loop.set_control_flow(ControlFlow::WaitUntil(
                    self.last_frame + Duration::from_millis(1000 / self.config.fps),
                ));
            }
        }
    }

    let event_loop = EventLoop::new()?;
    let mut app = App {
        window: None,
        renderer: None,
        simulation: sim,
        last_frame: Instant::now(),
        config,
    };

    event_loop.run_app(&mut app)?;

    Ok(())
}
