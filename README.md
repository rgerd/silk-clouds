<h1 align="center">
  <img width="256" alt="clouds-thumbnail" src="https://github.com/rgerd/silk-clouds/assets/4724014/0eaa8581-237d-4967-9762-c1ec5d7f44c7">
  <p>silky clouds</p>
</h1>

Rendering a density function in real time using marching cubes without leaving the GPU.

Runs at 60 FPS generating up to around 2.8 million vertices + normals inside a voxel cube with side-length 100 (1 million voxels) in multiple passes on an M2 Macbook Air.

To try it out, clone the repository and run `cargo run --release` from the root directory. Make sure you have [the Rust toolchain](https://www.rust-lang.org/learn/get-started) installed.

## How it works
This technique samples a simplex noise function into a 3D texture, runs [marching cubes](https://en.wikipedia.org/wiki/Marching_cubes) on that texture, filling a buffer with vertex data, and then uses an [indirect draw call](https://toji.dev/webgpu-best-practices/indirect-draws.html) to draw the generated vertex data.

Because the geometry generated by marching cubes is dynamically-sized, this technique typically requires a CPU-side copy to set up the render pass. However, by creating a vertex buffer of (tunable) amortized size and generating an indirect draw call buffer, we can do everything from the GPU.

#### Benefits
* Saves an expensive GPU-CPU round-trip. Graphics programmers are allergic to these.
* Simplifies the rendering process as no extra synchronization needs to happen between stages of the generation and rendering process from the removed copies.
* Sparse sampling of the density function with high-quality interpolation. When asked to render the isosurface of a density function, some may think to use [ray marching](https://typhomnt.github.io/teaching/ray_tracing/raymarching_intro/). However, this technique yields similar results thanks to the normal calculation step, with far fewer queries to the density function--leveraging the triangle rendering pipeline smart people have spent decades optimizing.

#### Limitations
* No CPU-side collision detection. Because the GPU holds the density function and data isn't copied to the CPU, this approach doesn't readily lend itself to real-time collision detection. Two options for getting around this are (1) doing collision detection on the GPU and (2) reading back the geometry buffer to the CPU, but maybe only every other frame if you still want some perf benefit.
* Could require lots of GPU memory and bandwidth, depending on what you're rendering. If you're rendering a very complex surface with lots of holes or extremely high resolution that can't be emulated by generated normals then this may not be the right technique. In fact, ray marching may be the better option here.

## Resources
* [NVIDIA GPU Gems 3: Chapter 1. Generating Complex Procedural Terrains Using the GPU](https://developer.nvidia.com/gpugems/gpugems3/part-i-geometry/chapter-1-generating-complex-procedural-terrains-using-gpu)
* [Coding Adventure: Marching Cubes by Sebastian Lague](https://www.youtube.com/watch?v=M3iI2l0ltbE)
* [Learn WGPU](https://sotrh.github.io/learn-wgpu/)
