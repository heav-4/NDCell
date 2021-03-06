use glium::glutin;
use imgui::{Context, FontSource};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use log::warn;
use send_wrapper::SendWrapper;
use std::cell::RefCell;
use std::time::Instant;

use ndcell_core::*;

use super::clipboard_compat::*;
use super::gridview::*;

lazy_static! {
    static ref EVENTS_LOOP: SendWrapper<RefCell<glutin::EventsLoop>> =
        SendWrapper::new(RefCell::new(glutin::EventsLoop::new()));
    pub static ref DISPLAY: SendWrapper<glium::Display> = SendWrapper::new({
        let wb = glutin::WindowBuilder::new().with_title(super::TITLE.to_owned());
        let cb = glutin::ContextBuilder::new().with_vsync(true);
        glium::Display::new(wb, cb, &EVENTS_LOOP.borrow()).expect("Failed to initialize display")
    });
}

const GOSPER_GLIDER_GUN_SYNTH_RLE: &str = "
#CXRLE Gen=-31
x = 47, y = 14, rule = Life
16bo30b$16bobo16bo11b$16b2o17bobo9b$obo10bo21b2o10b$b2o11b2o31b$bo11b
2o32b3$10b2o20b2o13b$11b2o19bobo9b3o$10bo21bo11bo2b$27bo17bob$27b2o18b
$26bobo!
";

fn make_default_gridview() -> GridView {
    let mut automaton = Automaton2D::from_rle(GOSPER_GLIDER_GUN_SYNTH_RLE).unwrap_or_else(|_| {
        warn!("Unable to parse default pattern; using empty pattern instead");
        Default::default()
    });
    automaton.set_sim(Simulation::from(rule::LIFE));
    GridView::from(automaton)
}

/// Display the main application window.
pub fn show_gui() {
    let display = &**DISPLAY;

    // Initialize runtime data.
    let mut config = super::config::Config::default();
    let mut gridview = make_default_gridview();
    let mut main_window = super::windows::MainWindow::default();
    let mut input_state = super::input::State::default();

    // Initialize imgui.
    let mut imgui = Context::create();
    imgui.set_clipboard_backend(Box::new(ClipboardCompat));
    imgui.set_ini_filename(None);
    let mut platform = WinitPlatform::init(&mut imgui);
    let gl_window = display.gl_window();
    let window = gl_window.window();
    // Using any other HiDpiMode::Default or HiDpiMode::Rounded makes window
    // borders and other things fuzzy for me (DPI = 1.5 in my setup) and
    // scaling can otherwise be accomplished by changing the font size, with
    // a MUCH crisper result.
    platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Default);
    config.gfx.dpi = platform.hidpi_factor(); // Fetch the DPI we want.
    platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Locked(1.0));

    // Initialize imgui fonts.
    let font_size = 16.0 * config.gfx.dpi as f32;
    imgui.fonts().add_font(&[FontSource::TtfData {
        data: include_bytes!("../resources/font/NotoSans-Regular.ttf"),
        size_pixels: font_size,
        config: None,
    }]);

    // Initialize imgui renderer.
    let mut renderer = Renderer::init(&mut imgui, display).expect("Failed to initialize renderer");

    // Main loop
    let mut last_frame_time = Instant::now();
    let mut closed = false;
    while !closed {
        let imgui_io = imgui.io_mut();
        platform
            .prepare_frame(imgui_io, &window)
            .expect("Failed to start frame");
        last_frame_time = imgui_io.update_delta_time(last_frame_time);

        let mut input_frame = input_state.frame(&mut config, &gridview, &imgui_io);

        EVENTS_LOOP.borrow_mut().poll_events(|ev| {
            // Let imgui handle events.
            platform.handle_event(imgui_io, &window, &ev);
            // Handle events for the grid view.
            input_frame.handle_event(&ev);
            // Handle events ourself.
            match ev {
                glutin::Event::WindowEvent { event, .. } => match event {
                    // Handle window close event.
                    glutin::WindowEvent::CloseRequested => closed = true,
                    _ => (),
                },
                _ => (),
            }
        });

        input_frame.finish();

        let ui = imgui.frame();
        main_window.build(&ui, &mut config, &gridview);

        gridview.do_frame(&config);

        let mut target = display.draw();

        match &mut gridview {
            GridView::View2D(view2d) => {
                view2d.render(
                    &config,
                    &mut target,
                    View2DRenderParams {
                        cursor_pos: input_state.get_cursor_pos(),
                    },
                );
            }
            GridView::View3D(_view3d) => (),
        };

        platform.prepare_render(&ui, &window);
        let draw_data = ui.render();
        renderer
            .render(&mut target, draw_data)
            .expect("Rendering failed");

        target.finish().expect("Failed to swap buffers");
    }
}
