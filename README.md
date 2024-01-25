# Underwater World

Infinite explorable underwater world created using Rust and WGPU using marching cubes and 3D perlin noise.

## TODO

- 3d fish/boids
    - Performance
        - 3d space partitioning
            - Is this actually helpful??
        - The slowest part is actually the raycasting/wall collision checks
            - It runs great otherwise
        - Is it fine actually?
    - Don't reset acceration between frames?
    - Specicies
        - Fix red and blue `vt`s?
            - I think caused by the `.jpg` instead of `.png`?
- Resizing
- Preformance
    - What is making it slow on the web??
- Better terrain generation
    - Coloring terrain
    - More phsyically plausible
- Propeller bubbles
- Shader/lighting effects
    - Fog
    - Lighting
    - Darker the deeper
        - Scale clear color/fog color with sub depth
        - Make sure the html background also updates
    - If no wall collisions then void plane to make it obvious you are below
- Collisions with walls?
    - Sub is hard to control + might be frustrating to get stuck in dead end caves
    - But the upside is you just go through walls
- Web build

## Controls

- W | up / S | down = pitch up / down
- A | left / D | right = pitch left / right
- Q | pgUp / E | pgDown = roll left / right
- Space / control = speed up / down
- R | Enter = reset submarine

## Assets

- Submarine: antonmoek - https://www.cgtrader.com/free-3d-models/vehicle/other/low-poly-cartoon-submarine
- Red fish: 3DRPolyFactory - https://www.cgtrader.com/free-3d-models/animals/fish/fish3d-v1
- Green fish: 3DRPolyFactory - https://www.cgtrader.com/free-3d-models/animals/fish/fish-3d-6a34c6e0-dff2-4375-9257-469577d423cd
- Blue fish: 3DRPolyFactory - https://www.cgtrader.com/free-3d-models/animals/fish/bluegill-886e1016-26b4-49c2-a594-799da26c1ce7

## Resources

- todo!()
- https://github.com/albertomelladoc/Fish-Animation/blob/master/FishAnimation.shader
