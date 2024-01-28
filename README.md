# Underwater World

Infinite explorable underwater world created using Rust and WGPU using marching cubes and 3D perlin noise.

## TODO

- Preformance
    - What are actually the slow parts?
    - Chunk generation
        - Issue: chunks are not created fast enough, can see them "pop" in
        - Attempted solution: start generating them before they are in the view dist?
            - Still not fast enough at 60 FPS
            - Sort of works when `GENERATION_DIST - VIEW_DIST >= 2`
    - "Downscale" chunks
        - They are 16x16x16 in world space but only 12x12x12 in local space
        - Shouldn't actually effect too much except for the boid wall avoidance
- 3d fish/boids
    - Performance
        - 3d space partitioning
            - Is this actually helpful??
        - The slowest part is actually the raycasting/wall collision checks
            - Might be a faster way to early exit
        - Is it fine actually?
    - Smoother wall avoidence
        - Don't reset acceration between frames?
            - Or don't reset `wall_avoidence_acceleration` each frame?
            - And have it decay to 0 over ~1 second?
    - Specicies
        - Fix red and blue `vt`s?
            - I think caused by the `.jpg` instead of `.png`?
- Resizing
- Better terrain generation
    - And coloring terrain
    - And more phsyically plausible
- Propeller bubbles?
- Shader/lighting effects
    - Fog
    - Lighting
    - Darker the deeper
        - Scale clear color/fog color with sub depth
        - Make sure the html background also updates
    - If no wall collisions then void plane to make it obvious you are below
- Web build

## Controls

- Change pitch: WASD or arrow keys
- Roll: Q/E or pgUp/pgDown
- Speed up: space
- Slow down: control
- Reset submarine: R or enter

## Assets

- Submarine: antonmoek - https://www.cgtrader.com/free-3d-models/vehicle/other/low-poly-cartoon-submarine
- Red fish: 3DRPolyFactory - https://www.cgtrader.com/free-3d-models/animals/fish/fish3d-v1
- Green fish: 3DRPolyFactory - https://www.cgtrader.com/free-3d-models/animals/fish/fish-3d-6a34c6e0-dff2-4375-9257-469577d423cd
- Blue fish: 3DRPolyFactory - https://www.cgtrader.com/free-3d-models/animals/fish/bluegill-886e1016-26b4-49c2-a594-799da26c1ce7

## Resources

- todo!()
- https://github.com/albertomelladoc/Fish-Animation/blob/master/FishAnimation.shader
