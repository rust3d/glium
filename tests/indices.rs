extern crate glutin;
#[macro_use]
extern crate glium;

use std::default::Default;
use glium::{index, Surface};

mod support;

fn build_program(display: &glium::Display) -> glium::Program {
    glium::Program::from_source(display,
        "
            #version 110

            attribute vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
        "
            #version 110

            void main() {
                gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
        None).unwrap()
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

#[test]
fn triangles_list_cpu() {    
    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, vec![
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]);

    let indices = glium::index::TrianglesList(vec![0u16, 1, 2, 2, 1, 3]);

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vb, &indices, &program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();
    target.finish();

    let data: Vec<Vec<(u8, u8, u8)>> = display.read_front_buffer();

    assert_eq!(data[0][0], (255, 0, 0));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0));

    display.assert_no_error();
}

#[test]
fn triangle_strip_cpu() {
    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, vec![
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]);

    let indices = glium::index::TriangleStrip(vec![0u16, 1, 2, 3]);

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vb, &indices, &program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();
    target.finish();

    let data: Vec<Vec<(u8, u8, u8)>> = display.read_front_buffer();

    assert_eq!(data[0][0], (255, 0, 0));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0));

    display.assert_no_error();
}

#[test]
fn triangle_fan_cpu() {
    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, vec![
        Vertex { position: [0.0,  0.0] },
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]);

    let indices = glium::index::TriangleFan(vec![0u16, 1, 2, 4, 3, 1]);

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vb, &indices, &program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();
    target.finish();

    let data: Vec<Vec<(u8, u8, u8)>> = display.read_front_buffer();

    assert_eq!(data[0][0], (255, 0, 0));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0));

    display.assert_no_error();
}

#[test]
fn triangles_list_gpu() {
    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, vec![
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]);

    let indices = glium::index::TrianglesList(vec![0u16, 1, 2, 2, 1, 3]);
    let indices = glium::IndexBuffer::new(&display, indices);

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vb, &indices, &program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();
    target.finish();

    let data: Vec<Vec<(u8, u8, u8)>> = display.read_front_buffer();

    assert_eq!(data[0][0], (255, 0, 0));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0));

    display.assert_no_error();
}

#[test]
fn triangle_strip_gpu() {
    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, vec![
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]);

    let indices = glium::index::TriangleStrip(vec![0u16, 1, 2, 3]);
    let indices = glium::IndexBuffer::new(&display, indices);

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vb, &indices, &program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();
    target.finish();

    let data: Vec<Vec<(u8, u8, u8)>> = display.read_front_buffer();

    assert_eq!(data[0][0], (255, 0, 0));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0));
    
    display.assert_no_error();
}

#[test]
fn triangle_fan_gpu() {
    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, vec![
        Vertex { position: [0.0,  0.0] },
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]);

    let indices = glium::index::TriangleFan(vec![0u16, 1, 2, 4, 3, 1]);
    let indices = glium::IndexBuffer::new(&display, indices);

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vb, &indices, &program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();
    target.finish();

    let data: Vec<Vec<(u8, u8, u8)>> = display.read_front_buffer();

    assert_eq!(data[0][0], (255, 0, 0));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0));
    
    display.assert_no_error();
}

#[test]
fn get_primitives_type() {
    let display = support::build_display();

    let indices = glium::index::TriangleStrip(vec![0u16, 1, 2, 3]);
    let indices = glium::IndexBuffer::new(&display, indices);

    assert_eq!(indices.get_primitives_type(), glium::index::PrimitiveType::TriangleStrip);

    display.assert_no_error();
}

#[test]
fn get_indices_type_u8() {
    let display = support::build_display();

    let indices = glium::index::TriangleStrip(vec![0u8, 1, 2, 3]);
    let indices = glium::IndexBuffer::new(&display, indices);

    assert_eq!(indices.get_indices_type(), glium::index::IndexType::U8);

    display.assert_no_error();
}

#[test]
fn get_indices_type_u16() {
    let display = support::build_display();

    let indices = glium::index::TriangleStrip(vec![0u16, 1, 2, 3]);
    let indices = glium::IndexBuffer::new(&display, indices);

    assert_eq!(indices.get_indices_type(), glium::index::IndexType::U16);

    display.assert_no_error();
}

#[test]
fn get_indices_type_u32() {
    let display = support::build_display();

    let indices = glium::index::TriangleStrip(vec![0u32, 1, 2, 3]);
    let indices = glium::IndexBuffer::new(&display, indices);

    assert_eq!(indices.get_indices_type(), glium::index::IndexType::U32);

    display.assert_no_error();
}

#[test]
fn triangles_list_noindices() {    
    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, vec![
        Vertex { position: [-1.0,  1.0] },
        Vertex { position: [ 1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] },
        Vertex { position: [-1.0, -1.0] },
        Vertex { position: [ 1.0,  1.0] },
        Vertex { position: [ 1.0, -1.0] },
    ]);

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vb, &index::NoIndices(index::PrimitiveType::TrianglesList),
                &program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();
    target.finish();

    let data: Vec<Vec<(u8, u8, u8)>> = display.read_front_buffer();

    assert_eq!(data[0][0], (255, 0, 0));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0));

    display.assert_no_error();
}

#[test]
fn triangle_strip_noindices() {
    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, vec![
        Vertex { position: [-1.0,  1.0] },
        Vertex { position: [ 1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] },
        Vertex { position: [ 1.0, -1.0] },
    ]);

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vb, &index::NoIndices(index::PrimitiveType::TriangleStrip),
                &program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();
    target.finish();

    let data: Vec<Vec<(u8, u8, u8)>> = display.read_front_buffer();

    assert_eq!(data[0][0], (255, 0, 0));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0));

    display.assert_no_error();
}

#[test]
fn triangle_fan_noindices() {
    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, vec![
        Vertex { position: [ 0.0,  0.0] },
        Vertex { position: [-1.0,  1.0] },
        Vertex { position: [ 1.0,  1.0] },
        Vertex { position: [ 1.0, -1.0] },
        Vertex { position: [-1.0, -1.0] },
        Vertex { position: [-1.0,  1.0] },
    ]);

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vb, &index::NoIndices(index::PrimitiveType::TriangleFan),
                &program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();
    target.finish();

    let data: Vec<Vec<(u8, u8, u8)>> = display.read_front_buffer();

    assert_eq!(data[0][0], (255, 0, 0));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0));

    display.assert_no_error();
}

#[test]
fn empty_index_buffer() {
    let display = support::build_display();

    let indices = glium::index::TriangleFan(Vec::<u16>::new());
    let _indices = glium::IndexBuffer::new(&display, indices);

    display.assert_no_error();
}

#[test]
fn indexbuffer_slice_out_of_range() {    
    let display = support::build_display();

    let indices = glium::index::TrianglesList(vec![0u16, 1, 2, 2, 1, 3]);
    let indices = glium::IndexBuffer::new(&display, indices);

    assert!(indices.slice(5 .. 8).is_none());
    assert!(indices.slice(2 .. 11).is_none());
    assert!(indices.slice(12 .. 13).is_none());

    display.assert_no_error();
}

#[test]
fn indexbuffer_slice_draw() {    
    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, vec![
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]);

    let indices = glium::index::TrianglesList(vec![0u16, 3, 2, 0, 1, 3]);
    let indices = glium::IndexBuffer::new(&display, indices);

    let texture1 = support::build_renderable_texture(&display);
    texture1.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture1.as_surface().draw(&vb, &indices.slice(3 .. 6).unwrap(), &program,
                &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8)>> = texture1.read();
    assert_eq!(data[0][0], (0, 0, 0));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0));


    let texture2 = support::build_renderable_texture(&display);
    texture2.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture2.as_surface().draw(&vb, &indices.slice(0 .. 3).unwrap(), &program,
                &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8)>> = texture2.read();
    assert_eq!(data[0][0], (255, 0, 0));
    assert_eq!(data.last().unwrap().last().unwrap(), &(0, 0, 0));


    display.assert_no_error();
}
