# Underwater World

Infinite explorable underwater world created using Rust and WGPU using marching cubes and 3D perlin noise.

## TODO

- Better terrain generation
    - + Coloring terrain
    - More phsyically plausible
- Resizing
- Collisions with walls?
    - Sub is hard to control + might be frustrating to get stuck in dead end caves
    - But the upside is you just go through walls
- 3d fish/boids
    - Obstacle avoidance
    - Preformance: only exist in nearby chunks
- Text
    - FPS
    - Posistion
    - Bearing
- Propeller bubbles
- Shader/lighting effects
    - Fog
    - Lighting
    - Darker the deeper
    - If no wall collisions then void plane to make it obvious you are below
 - Web build

## Controls

- W | up / S | down = pitch up / down
- A | left / D | right = pitch left / right
- Q | pgUp / E | pgDown = roll left / right
- Space / control = speed up / down
- R | Enter = reset submarine

## Assets

- Submarine: antonmoek - https://www.cgtrader.com/free-3d-models/vehicle/other/low-poly-cartoon-submarine

## Resources

- todo!()