use glium::glutin::{self, Event, WindowEvent};
use glium::{Display, Surface};
use imgui::{Context, FontConfig, FontGlyphRanges, FontSource, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::Instant;
use winit::dpi::LogicalSize;

const SEARCH_SIZE: LogicalSize = LogicalSize {
    width: 680f64,
    height: 60f64
};

const RESULTS_SIZE: LogicalSize = LogicalSize {
    width: 680f64,
    height: 500f64
};

pub struct System {
    pub events_loop: glutin::EventsLoop,
    pub display: glium::Display,
    pub imgui: Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
    pub font_size: f32,
}

pub fn init(title: &str) -> System {
    let title = match title.rfind('/') {
        Some(idx) => title.split_at(idx + 1).1,
        None => title,
    };
    let events_loop = glutin::EventsLoop::new();
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let builder = glutin::WindowBuilder::new()
        .with_title(title)
        .with_dimensions(glutin::dpi::LogicalSize::new(680f64, 60f64))
        .with_resizable(false)
        // .with_transparency(true)
        .with_always_on_top(true)
        .with_decorations(false);
    let display =
        Display::new(builder, context, &events_loop).expect("Failed to initialize display");

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);

    let mut platform = WinitPlatform::init(&mut imgui);
    {
        let gl_window = display.gl_window();
        let window = gl_window.window();
        platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);
    }

    let hidpi_factor = platform.hidpi_factor();
    let font_size = (13.0 * hidpi_factor) as f32;
    imgui.fonts().add_font(&[
        FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: font_size,
                ..FontConfig::default()
            }),
        },
        FontSource::TtfData {
            data: include_bytes!("../../../assets/System San Francisco Display Regular.ttf"),
            size_pixels: font_size,
            config: Some(FontConfig {
                rasterizer_multiply: 1.75,
                glyph_ranges: FontGlyphRanges::japanese(),
                ..FontConfig::default()
            }),
        },
    ]);

    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    let renderer = Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

    System {
        events_loop,
        display,
        imgui,
        platform,
        renderer,
        font_size,
    }
}

#[derive(Debug)]
pub struct RunState {
    pub status: bool,
    pub showing_results: bool
}

impl System {
    pub fn main_loop<F: FnMut(&mut bool, &mut Ui) -> RunState>(self, mut run_ui: F) {
        let System {
            mut events_loop,
            display,
            mut imgui,
            mut platform,
            mut renderer,
            ..
        } = self;
        let gl_window = display.gl_window();
        let window = gl_window.window();
        let mut last_frame = Instant::now();
        let mut run = true;

        while run {
            events_loop.poll_events(|event| {
                platform.handle_event(imgui.io_mut(), &window, &event);

                if let Event::WindowEvent { event, .. } = event {
                    if let WindowEvent::CloseRequested = event {
                        run = false;
                    }
                }
            });

            let io = imgui.io_mut();
            platform
                .prepare_frame(io, &window)
                .expect("Failed to start frame");
            last_frame = io.update_delta_time(last_frame);
            let mut ui = imgui.frame();
            let run_state = run_ui(&mut run, &mut ui);

            if !run_state.status {
                break;
            }

            if run_state.showing_results {
                if window.get_inner_size().unwrap() == SEARCH_SIZE {
                    window.set_inner_size(RESULTS_SIZE);
                }
            }
            else if !run_state.showing_results {
                if window.get_inner_size().unwrap() == RESULTS_SIZE {
                    window.set_inner_size(SEARCH_SIZE);
                }
            }

            let mut target = display.draw();
            target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
            platform.prepare_render(&ui, &window);
            let draw_data = ui.render();
            renderer
                .render(&mut target, draw_data)
                .expect("Rendering failed");
            target.finish().expect("Failed to swap buffers");
        }
    }
}
