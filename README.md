# Underwater World

Infinite explorable underwater world created using Rust and WGPU using marching cubes and 3D perlin noise.

## TODO

- 3d fish/boids
    - Wall avoidence
        - Unity `Physics.SphereCast` like raycasts?
        - Pick:
            - `RAY_DIRECTION_COUNT`
        - It is missing a `* delta`?
            - `WALL_FORCE_MULT`
        - Should avoidance_rays be the same for every boid?
        - On intersect checks, should do match and a `t` vs `WALL_RANGE as f32`?
            - Or just `t.is_some()`/`t.is_none()`?
    - Wrapping
        - Try to stay within the view/generation view frustrums?
            - Would be able to lower the number of boids and have the same effect
    - Performance
        - The slowest part is actually the raycasting/wall collision checks
            - Might be a faster way to early exit
                - Early dist check before intersection check?
                - Know we only want the closest t, look for that first?
- Preformance
    - What are actually the slow parts?
    - Super prio on chunks that are in the view frustrum?
        - Set up a prio struct to use for the generation sort
            - dist, z, gen frust count, view frust count
    - If have "extra preformance"
        - Bigger view distance (chunks + fog)
        - Build chunks faster/slower?
        - Boids
            - More boids
            - More wall avoidence rays
            - Higher wall avoidance range
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
        - View frustrum culling
        - Chunk generation order
            - Generate frustrum prio
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
    - Fog, darker/deeper, fish swim animation

## Assets

- Submarine: antonmoek - https://www.cgtrader.com/free-3d-models/vehicle/other/low-poly-cartoon-submarine
- Red fish: 3DRPolyFactory - https://www.cgtrader.com/free-3d-models/animals/fish/fish3d-v1
- Green fish: 3DRPolyFactory - https://www.cgtrader.com/free-3d-models/animals/fish/fish-3d-6a34c6e0-dff2-4375-9257-469577d423cd
- Blue fish: 3DRPolyFactory - https://www.cgtrader.com/free-3d-models/animals/fish/bluegill-886e1016-26b4-49c2-a594-799da26c1ce7

## Resources

- todo!()
- https://github.com/albertomelladoc/Fish-Animation/blob/master/FishAnimation.shader
