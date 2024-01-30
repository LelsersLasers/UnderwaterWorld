# Underwater World

Infinite explorable underwater world created using Rust and WGPU using marching cubes and 3D perlin noise.

## TODO

- Preformance
    - What are actually the slow parts?
    - Chunk ordering
        - Either only do or do first the chunks that are in the right direction
            - No point in generating or rendering chunks that are behind you
            - Plus turn rate is fairly slow, so would have time to adjust?
        - For every point on the edge of a chunk:
            - If `camera.uniform.view_proj * chunk` is contained in the normalized device coordinates
                - Then: give it priority
    - If have "extra preformance"
        - Bigger view distance (chunks + fog)
        - Build chunks faster/slower?
        - More rays for boid wall avoidence?
        - More boids?
- 3d fish/boids
    - Performance
        - 3d space partitioning
            - Is this actually needed?
            - Will my implementation be faster than just checking every boid?
        - The slowest part is actually the raycasting/wall collision checks
            - Might be a faster way to early exit
                - Early dist check before intersection check?
                - Know we only want the closest t, look for that first?
        - Is it fine actually?
    - Wall avoidence
        - Smoother wall avoidence
            - Don't reset acceration between frames?
                - Or don't reset `wall_avoidence_acceleration` each frame?
                - And have it decay to 0 over ~1 second?
        - Better system
            - Multiple rays?
            - Perpendicular to the normal?
    - Specicies
        - Fix red and blue `vt`s?
            - I think caused by the `.jpg` instead of `.png`?
- Better terrain generation
    - And coloring terrain
    - And more phsyically plausible
- Propeller bubbles?
- Shader/lighting effects
    - Fog
    - Lighting?
    - Darker the deeper
        - Scale clear color/fog color with sub depth
    - Void plane to make it obvious you are looking below the terrain
- Web build

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
        - Downscaling
        - Smart sorting
        - Blank chunk (+ early generation check)
        - Frustrum culling
- Marching Cubes
- Fish
    - 3d boids
    - Wall avoidence
    - Wrapping system
- Terrain Generation/Perlin Noise
    - 3d multi-octave perlin noise
- Shader/Lighting Effects
    - Fog, darker/deeper, fish swim animation

## Assets

- Submarine: antonmoek - https://www.cgtrader.com/free-3d-models/vehicle/other/low-poly-cartoon-submarine
- Red fish: 3DRPolyFactory - https://www.cgtrader.com/free-3d-models/animals/fish/fish3d-v1
- Green fish: 3DRPolyFactory - https://www.cgtrader.com/free-3d-models/animals/fish/fish-3d-6a34c6e0-dff2-4375-9257-469577d423cd
- Blue fish: 3DRPolyFactory - https://www.cgtrader.com/free-3d-models/animals/fish/bluegill-886e1016-26b4-49c2-a594-799da26c1ce7

## Resources

- todo!()
- https://github.com/albertomelladoc/Fish-Animation/blob/master/FishAnimation.shader
