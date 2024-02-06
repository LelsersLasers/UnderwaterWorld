# Underwater World

Infinite explorable underwater world created using Rust and WGPU using marching cubes and 3D perlin noise.

## TODO

- README.md write up
- itch.io page

## Controls

- Change pitch: WASD or arrow keys
- Roll: Q/E or pgUp/pgDown
- Speed up: space
- Slow down: control
- Reset submarine: R or enter

## Notes

- todo!()
- Performance
    - Chunk generation
        - Split across multiple frames
        - Index buffers
        - Downscaling
        - Smart sorting
        - Blank chunk (+ early generation check)
        - View frustrum culling
        - Chunk generation order
    - Spatial paritioning
- Marching Cubes
- Fish
    - 3d boids
    - Wall avoidence
        - 3d points on sphere
        - Raycasting
    - Wrapping system
- Terrain Generation/Perlin Noise
    - 3d multi-octave perlin noise
- Shader/Lighting Effects
    - Fog, darker/deeper, fish swim animation, sub light

## Assets

- Submarine: antonmoek - https://www.cgtrader.com/free-3d-models/vehicle/other/low-poly-cartoon-submarine
- Red fish: 3DRPolyFactory - https://www.cgtrader.com/free-3d-models/animals/fish/fish3d-v1
- Green fish: 3DRPolyFactory - https://www.cgtrader.com/free-3d-models/animals/fish/fish-3d-6a34c6e0-dff2-4375-9257-469577d423cd
- Blue fish: 3DRPolyFactory - https://www.cgtrader.com/free-3d-models/animals/fish/bluegill-886e1016-26b4-49c2-a594-799da26c1ce7

## Resources

- https://sotrh.github.io/learn-wgpu/
- Marching Cubes/World Generation
    - https://www.youtube.com/watch?v=M3iI2l0ltbE
    - https://paulbourke.net/geometry/polygonise/
    - https://developer.nvidia.com/gpugems/gpugems3/part-i-geometry/chapter-1-generating-complex-procedural-terrains-using-gpu
    - https://people.eecs.berkeley.edu/~jrs/meshpapers/LorensenCline.pdf
    - https://www.youtube.com/watch?v=YyVAaJqYAfE
    - https://www.youtube.com/watch?v=TZFv493D7jo
    - https://www.youtube.com/watch?v=4O0_-1NaWny
- Boids
    - https://stackoverflow.com/questions/9600801/evenly-distributing-n-points-on-a-sphere/44164075#44164075
    - https://www.youtube.com/watch?v=bqtqltqcQhw
    - https://natureofcode.com/book/chapter-6-autonomous-agents/
    - https://www.red3d.com/cwr/boids/
    - https://www.youtube.com/watch?v=gpc7u3331oQ
    - https://eater.net/boids
    - https://github.com/miorsoft/VB6-3D-Flocking-Boids
    - https://github.com/albertomelladoc/Fish-Animation/blob/master/FishAnimation.shader
- Techniques from old projects:
    - https://lelserslasers.itch.io/boids
    - https://lelserslasers.itch.io/3d-cellular-automata-wgpu-rust
    - https://lelserslasers.itch.io/minecraft