use std::{ops::AddAssign, sync::Arc};

use rug::Float;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{self, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::wgpu::WgpuState;

#[derive(Default)]
pub struct App<'a> {
    state: Option<WgpuState<'a>>,
    window: Option<Arc<Window>>,

    cursor_position: Option<PhysicalPosition<f64>>,
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = event_loop
                .create_window(Window::default_attributes().with_title("Mandelbrot"))
                .expect("Unable to create window");
            let window = Arc::new(window);

            self.window = Some(window.clone());

            let wgpu_state = pollster::block_on(WgpuState::new(window.clone()));
            self.state = Some(wgpu_state);

            println!("Window created.");
            println!("Controls:");
            println!("  - Scroll: Zoom in/out");

            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Close requested, exiting.");
                event_loop.exit();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = Some(position);
            }
            // Handle Scrolling (Zoom)
            WindowEvent::MouseWheel { delta, .. } => {
                let cursor_pos = self.cursor_position;
                if let Some(state) = &mut self.state {
                    Self::update_camera(state, cursor_pos, delta);

                    let log_z = state
                        .uniform_data
                        .fractal_state
                        .zoom
                        .clone()
                        .log10()
                        .to_f32();
                    println!("Zoom: 10^{:.2}", log_z);

                    self.window.as_ref().unwrap().request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.state {
                    state.update();
                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(state) = &mut self.state {
                    state.resize(physical_size);
                }
            }
            _ => {}
        }
    }
}

impl<'a> App<'a> {
    fn update_camera(
        state: &mut WgpuState<'_>,
        cursor_position: Option<PhysicalPosition<f64>>,
        delta: event::MouseScrollDelta,
    ) {
        let old_zoom = state.uniform_data.fractal_state.zoom.clone();

        let zoom_factor = 1.15;
        let mut zoom_mult = Float::with_val(128, zoom_factor);

        // Calculate Zoom Multiplier
        match delta {
            event::MouseScrollDelta::LineDelta(_, y) => {
                if y < 0.0 {
                    zoom_mult = Float::with_val(128, 1.0) / zoom_mult;
                }
            }
            event::MouseScrollDelta::PixelDelta(pos) => {
                if pos.y < 0.0 {
                    zoom_mult = Float::with_val(128, 1.0) / zoom_mult;
                }
            }
        }

        let new_zoom = Float::with_val(128, &old_zoom * &zoom_mult);

        // Handle "Zoom Towards Cursor"
        if let Some(pos) = cursor_position {
            let width = state.config.width as f64;
            let height = state.config.height as f64;
            let aspect = width / height;

            let ndc_x = (pos.x / width) * 2.0 - 1.0;
            let ndc_y = 1.0 - (pos.y / height) * 2.0;

            let mouse_vec_x = ndc_x * aspect;
            let mouse_vec_y = ndc_y;

            // Calculate "Zoom Difference"
            let one = Float::with_val(128, 1.0);
            let inv_old = Float::with_val(128, &one / &old_zoom);
            let inv_new = Float::with_val(128, &one / &new_zoom);
            let zoom_diff = inv_old - inv_new;

            let shift_x = Float::with_val(128, mouse_vec_x) * &zoom_diff;
            let shift_y = Float::with_val(128, mouse_vec_y) * &zoom_diff;

            // Apply to High Precision Center
            state
                .uniform_data
                .fractal_state
                .camera
                .mut_real()
                .add_assign(&shift_x);
            state
                .uniform_data
                .fractal_state
                .camera
                .mut_imag()
                .add_assign(&shift_y);
        }

        state.uniform_data.fractal_state.zoom = new_zoom;
    }
}
