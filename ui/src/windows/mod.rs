use imgui::*;

use ndcell_core::{AsFVec, Dim, Dim2D, NdSimulate, X, Y};

mod simulation;

use crate::config::*;
use crate::gridview::*;
use simulation::SimulationWindow;

const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const BLUE: [f32; 4] = [0.0, 0.5, 1.0, 1.0];

#[derive(Debug, Default)]
pub struct MainWindow {
    simulation: SimulationWindow,
}
impl MainWindow {
    /// Builds the main window.
    pub fn build(&mut self, ui: &imgui::Ui, config: &mut Config, gridview: &GridView) {
        Window::new(&ImString::new(crate::TITLE)).build(&ui, || {
            ui.text(format!("NDCell v{}", env!("CARGO_PKG_VERSION")));
            ui.text("");
            let fps = ui.io().framerate as usize;
            ui.text_colored(get_fps_color(fps), format!("Framerate = {} FPS", fps));
            if let GridView::View2D(view2d) = gridview {
                let total_update_ms: f64 = view2d
                    .last_sim_times
                    .iter()
                    .map(|duration| duration.as_secs_f64())
                    .sum();
                if total_update_ms == 0.0 {
                    ui.text("");
                } else {
                    let avg_update_ms = total_update_ms / view2d.last_sim_times.len() as f64;
                    let ups = (1.0 / avg_update_ms) as usize;
                    ui.text_colored(get_fps_color(ups), format!("Max sim speed = {} UPS", ups));
                }
                if view2d.is_drawing {
                    ui.text_colored(BLUE, "DRAWING");
                } else if view2d.is_waiting || view2d.is_running {
                    ui.text_colored(
                        if view2d.is_running {
                            if view2d.is_waiting {
                                RED
                            } else {
                                GREEN
                            }
                        } else {
                            YELLOW
                        },
                        "SIMULATING",
                    );
                } else {
                    ui.text("");
                }
            }
            ui.text("");
            ui.text(format!("Generations = {}", gridview.get_generation_count()));
            ui.text(format!("Population = {}", gridview.get_population()));
            ui.text("");
            match &gridview {
                GridView::View2D(view2d) => {
                    let Viewport2D { pos, offset, zoom } = &view2d.viewport;
                    ui.text(format!("Zoom = {}", zoom));
                    let total_pos = pos.as_fvec() + offset.clone();
                    for &ax in Dim2D::axes() {
                        let value = total_pos[ax];
                        if format!("{:.1}", value).ends_with("0") {
                            ui.text(format!("{} = {:.0}", ax.name(), value));
                        } else {
                            ui.text(format!("{} = {:.1}", ax.name(), value));
                        }
                    }
                    if let Some(hover_pos) = view2d.get_render_result(0).hover_pos.as_ref() {
                        ui.text(format!(
                            "Selected: X = {}, Y = {}",
                            hover_pos[X], hover_pos[Y]
                        ));
                    } else {
                        ui.text("");
                    }
                }
                _ => unimplemented!(),
            };
            ui.checkbox(im_str!("Simulation"), &mut self.simulation.is_visible);
        });
        self.simulation.build(ui, config, gridview)
    }
}

fn get_fps_color(fps: usize) -> [f32; 4] {
    if fps >= 58 {
        // Green
        [0.0, 1.0, 0.0, 1.0]
    } else if fps >= 29 {
        // Yellow
        [1.0, 1.0, 0.0, 1.0]
    } else {
        // Red
        [1.0, 0.0, 0.0, 1.0]
    }
}
