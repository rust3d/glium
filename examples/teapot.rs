extern crate glutin;

#[macro_use]
extern crate glium;

use glium::Surface;

mod support;

fn main() {
    use glium::DisplayBuild;

    // building the display, ie. the main object
    let display = glutin::WindowBuilder::new()
        .with_depth_buffer(24)
        .build_glium()
        .unwrap();

    // building the vertex and index buffers
    let vertex_buffer = support::load_wavefront(&display, include_bytes!("support/teapot.obj"));

    // the program
    let program = glium::Program::from_source(&display,
        // vertex shader
        "
            #version 110

            uniform mat4 persp_matrix;
            uniform mat4 view_matrix;

            attribute vec3 position;
            attribute vec3 normal;
            varying vec3 v_position;
            varying vec3 v_normal;

            void main() {
                v_position = position;
                v_normal = normal;
                gl_Position = persp_matrix * view_matrix * vec4(v_position * 0.005, 1.0);
            }
        ",

        // fragment shader
        "
            #version 110

            varying vec3 v_normal;

            const vec3 LIGHT = vec3(-0.2, 0.8, 0.1);

            void main() {
                float lum = max(dot(normalize(v_normal), normalize(LIGHT)), 0.0);
                vec3 color = (0.3 + 0.7 * lum) * vec3(1.0, 1.0, 1.0);
                gl_FragColor = vec4(color, 1.0);
            }
        ",

        // geometry shader
        None)
        .unwrap();

    //
    let mut camera = support::camera::CameraState::new();
    
    // the main loop
    support::start_loop(|| {
        camera.update();

        // building the uniforms
        let uniforms = uniform! {
            persp_matrix: camera.get_perspective(),
            view_matrix: camera.get_view(),
        };

        // draw parameters
        let params = glium::DrawParameters {
            depth_test: glium::DepthTest::IfLess,
            depth_write: true,
            .. std::default::Default::default()
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        target.draw(&vertex_buffer,
                    &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                    &program, &uniforms, &params).unwrap();
        target.finish();

        // polling and handling the events received by the window
        for event in display.poll_events() {
            match event {
                glutin::Event::Closed => return support::Action::Stop,
                ev => camera.process_input(&ev),
            }
        }

        support::Action::Continue
    });
}
