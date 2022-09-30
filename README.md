# Project Sandbox
Experimental rust falling sand simulation with long term goals of huge worlds and complex particle physics and interaction simulation.
For the latest working build, check the [Releases page](https://github.com/jakemcleman/project-sandbox/releases)

![Latest release status](https://github.com/jakemcleman/project-sandbox/actions/workflows/release.yml/badge.svg) ![Current main build/test](https://github.com/jakemcleman/project-sandbox/actions/workflows/rust.yml/badge.svg)

## Setup
Project is built using latest stable rust toolchain versions. On windows, just installing the rust/cargo toolchain is enough to get building.
On Linux there are a few extra things you may need, see [Bevy Linux Dependencies](https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md)

## Architecture
The project is currently split into 3 crates. 

### Gridmath
Contains the integer vector library used for this project. This includes the set of bounds helper functions that are used in the simulation, including iterators for traversing each integer coordinate within a bounding box.

### sandworld
Contains the core simulation, depends on gridmath. Uses [Rayon](https://github.com/rayon-rs/rayon) to multithread the simulation and provides an API to manipulate and help render it. Simulation is based on chunks, which each keep track of what areas need updating and process their own updates, allowing for movement into neighbors if needed. Each chunk is able to run its updates safely in parallel as long as no orthogonnaly or diagonally adjacent chunks are being updated at the same time.

### sandgame (top level executable)
Depends on the other 2 crates. Contains a [Bevy](https://github.com/bevyengine/bevy) app to run, render, and manipulate the simulation with basic UI. Renders each chunk as a sprite, using a color array produced by a chunk's render method.
