# Underwater World

Infinite explorable underwater world created using Rust and WGPU using marching cubes and 3D perlin noise.

## TODO

- 3d fish/boids
    - Obstacle avoidance
    - Not spawn in walls
    - Render
        - Animation: wiggle shader
        - Distance shader
    - Specicies
        - More than 2?
        - Fix red `vt`s?
        - Different type: ex: jellyfish
    - Some sort of "wall force" or downward force to keep them from around z ~= 0
    - Preformance: only exist in nearby chunks
        - Either wrap boids, delete far away boids, or cause them to turn back towards the sub
- Better terrain generation
    - + Coloring terrain
    - More phsyically plausible
- Resizing
- Collisions with walls?
    - Sub is hard to control + might be frustrating to get stuck in dead end caves
    - But the upside is you just go through walls
- Text
    - FPS
    - Posistion
    - Bearing
- Propeller bubbles
- Shader/lighting effects
    - Fog
    - Lighting
    - Darker the deeper
        - Scale clear color/fog color with sub depth
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
