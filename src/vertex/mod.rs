/*!
Contains everything related to vertex buffers.

The main struct is the `VertexBuffer`, which represents a buffer in the video memory,
containing a list of vertices.

In order to create a vertex buffer, you must first create a struct that represents each vertex,
and implement the `glium::vertex::Vertex` trait on it. The `implement_vertex!` macro helps you
with that.

```
# #[macro_use]
# extern crate glium;
# extern crate glutin;
# fn main() {
#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    texcoords: [f32; 2],
}

implement_vertex!(Vertex, position, texcoords);
# }
```

Next, build a `Vec` of the vertices that you want to upload, and pass it to
`VertexBuffer::new`.

```no_run
# let display: glium::Display = unsafe { ::std::mem::uninitialized() };
# #[derive(Copy, Clone)]
# struct Vertex {
#     position: [f32; 3],
#     texcoords: [f32; 2],
# }
# impl glium::vertex::Vertex for Vertex {
#     fn build_bindings() -> glium::vertex::VertexFormat {
#         unimplemented!() }
# }
let data = vec![
    Vertex {
        position: [0.0, 0.0, 0.4],
        texcoords: [0.0, 1.0]
    },
    Vertex {
        position: [12.0, 4.5, -1.8],
        texcoords: [1.0, 0.5]
    },
    Vertex {
        position: [-7.124, 0.1, 0.0],
        texcoords: [0.0, 0.4]
    },
];

let vertex_buffer = glium::vertex::VertexBuffer::new(&display, data);
```

*/
use std::iter::Chain;
use std::option::IntoIter;

pub use self::buffer::{VertexBuffer, VertexBufferAny, Mapping};
pub use self::buffer::{VertexBufferSlice, VertexBufferAnySlice};
pub use self::format::{AttributeType, VertexFormat};

mod buffer;
mod format;

/// Describes the source to use for the vertices when drawing.
#[derive(Clone)]
pub enum VerticesSource<'a> {
    /// A buffer uploaded in the video memory.
    ///
    /// The second and third parameters are the offset and length of the buffer.
    /// The fourth parameter tells whether or not this buffer is "per instance" (true) or
    // "per vertex" (false)
    VertexBuffer(&'a VertexBufferAny, usize, usize, bool),
}

/// Objects that can be used as vertex sources.
pub trait IntoVerticesSource<'a> {
    /// Builds the `VerticesSource`.
    fn into_vertices_source(self) -> VerticesSource<'a>;
}

impl<'a> IntoVerticesSource<'a> for VerticesSource<'a> {
    fn into_vertices_source(self) -> VerticesSource<'a> {
        self
    }
}

/// Marker that instructs glium that the buffer is to be used per instance.
pub struct PerInstance<'a>(VertexBufferAnySlice<'a>);

impl<'a> IntoVerticesSource<'a> for PerInstance<'a> {
    fn into_vertices_source(self) -> VerticesSource<'a> {
        match self.0.into_vertices_source() {
            VerticesSource::VertexBuffer(buf, off, len, false) => {
                VerticesSource::VertexBuffer(buf, off, len, true)
            },
            _ => unreachable!()
        }
    }
}

/// Objects that describe multiple vertex sources.
pub trait MultiVerticesSource<'a> {
    /// Iterator that enumerates each source.
    type Iterator: Iterator<Item = VerticesSource<'a>>;

    /// Iterates over the `VerticesSource`.
    fn iter(self) -> Self::Iterator;
}

impl<'a, T> MultiVerticesSource<'a> for T
    where T: IntoVerticesSource<'a>
{
    type Iterator = IntoIter<VerticesSource<'a>>;

    fn iter(self) -> IntoIter<VerticesSource<'a>> {
        Some(self.into_vertices_source()).into_iter()
    }
}

macro_rules! impl_for_tuple {
    ($t:ident) => (
        impl<'a, $t> MultiVerticesSource<'a> for ($t,)
            where $t: IntoVerticesSource<'a>
        {
            type Iterator = IntoIter<VerticesSource<'a>>;

            fn iter(self) -> IntoIter<VerticesSource<'a>> {
                Some(self.0.into_vertices_source()).into_iter()
            }
        }
    );

    ($t1:ident, $t2:ident) => (
        #[allow(non_snake_case)]
        impl<'a, $t1, $t2> MultiVerticesSource<'a> for ($t1, $t2)
            where $t1: IntoVerticesSource<'a>, $t2: IntoVerticesSource<'a>
        {
            type Iterator = Chain<<($t1,) as MultiVerticesSource<'a>>::Iterator,
                                  <($t2,) as MultiVerticesSource<'a>>::Iterator>;

            fn iter(self) -> Chain<<($t1,) as MultiVerticesSource<'a>>::Iterator,
                                   <($t2,) as MultiVerticesSource<'a>>::Iterator>
            {
                let ($t1, $t2) = self;
                Some($t1.into_vertices_source()).into_iter().chain(($t2,).iter())
            }
        }

        impl_for_tuple!($t2);
    );

    ($t1:ident, $($t2:ident),+) => (
        #[allow(non_snake_case)]
        impl<'a, $t1, $($t2),+> MultiVerticesSource<'a> for ($t1, $($t2),+)
            where $t1: IntoVerticesSource<'a>, $($t2: IntoVerticesSource<'a>),+
        {
            type Iterator = Chain<<($t1,) as MultiVerticesSource<'a>>::Iterator,
                                  <($($t2),+) as MultiVerticesSource<'a>>::Iterator>;

            fn iter(self) -> Chain<<($t1,) as MultiVerticesSource<'a>>::Iterator,
                                  <($($t2),+) as MultiVerticesSource<'a>>::Iterator>
            {
                let ($t1, $($t2),+) = self;
                Some($t1.into_vertices_source()).into_iter().chain(($($t2),+).iter())
            }
        }

        impl_for_tuple!($($t2),+);
    );
}

impl_for_tuple!(A, B, C, D, E, F, G);

/// Trait for structures that represent a vertex.
///
/// Instead of implementing this trait yourself, it is recommended to use the `implement_vertex!`
/// macro instead.
// TODO: this should be `unsafe`, but that would break the syntax extension
pub trait Vertex: Copy + Sized {
    /// Builds the `VertexFormat` representing the layout of this element.
    fn build_bindings() -> VertexFormat;
}

/// Trait for types that can be used as vertex attributes.
pub unsafe trait Attribute: Sized {
    /// Get the type of data.
    fn get_type() -> AttributeType;
}
