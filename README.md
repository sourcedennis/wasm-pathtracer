# Whitted-style Ray Tracing

Live demo: [https://raytracer.space/1/](https://raytracer.space/1/) (Definitely runs in Google Chrome)

This is a realtime whitted-style raytracer written in Rust, intended to be compiled into WebAssembly (WASM). The included TypeScript Web App periodically invokes the WASM raytracer, and provides a GUI to interact with it.

## 1. Running
Compiling the application yourself can be a bit troublesome, as compilers for *three* languages (TypeScript, Rust, Elm) need to be present.

> If no changes are made, I recommend using the live demo website listed at the top.

If you do compile it yourself, follow the steps below.
### Installing Elm
The Elm compiler can be downloaded through NPM. NPM is distributed with Node.js .

* [Node.js](https://nodejs.org/en/download/)

Then install Elm globally with:
```
npm install -g elm
```

### Installing Webpack
Webpack is used to compile and bundle all targets. The compile scripts are made for Webpack. Install Webpack globally as follows:
```
npm install -g webpack webpack-cli
```

### Installing Rust
The Rust compiler can be downloaded from:

* [Rust](https://www.rust-lang.org/)

### Compiling
Now all necessary system-wide tools should be available. Any other tools/libraries should be obtainable as part of the project's `npm` setup process. Inside the root of this project invoke:
```
npm install
```
This installs all local dependencies (into the `node_modules/` directory). To run the project in dev-mode, invoke:
```
npm run serve
```
This starts a HTTP server at [https://localhost:9000/](https://localhost:9000/), with the project running.

## 2. Interacting
> Note that the application may not run in all browsers. It surely works in Google Chrome, but (especially multi-threading) may not work in other browsers. Be aware of this.

The GUI buttons on the righthand-side are rather self-explanatory. Further controls are:

* WASD to *translate* the camera
* Arrow keys + PageUp + PageDown to *rotate* the camera

## 3. Code Navigation
The main architecture is divided between the TypeScript part and the Rust part. Rust implements the actual tracing of rays into (hardcoded) scenes. The TypeScript part handles the GUI (buttons + controls), and guides the multi-threading (as browsers can only multi-thread through WebWorkers). The general structure of both parts is outlines below:

### Rust Source
The Rust sourcecode is located in the `src` directory. Some important files are described below.

* `lib.rs` - The compilation entry point of the application (not really important)
* `wasm_interface.rs` - All *public-interface* functions. All functions that are called by TypeScript are placed here.
* `tracer.rs` - Contains code for tracing a single ray (recursively) into a scene
* `scenes.rs` - The hardcoded scenes
* `graphics/`
  * `scene.rs` - General scene description. Contains methods for tracing rays (and shadow rays) into the scene
  * `material.rs` - The material class (reflection, diffuse, refraction; potentially with textures)
  * `primitives/` - Contains all implemented primitives (each implements `Tracable` from `graphics/ray.rs`)

### TypeScript Source
The TypeScript sourcecode mainly deals with the GUI and the properly spawning of WebWorkers to attain proper multi-threading.

> Regarding multi-threading: The WASM module is compiled once, and then passed onto 8 WebWorkers. Each of these WebWorkers gets a random subset (partition) of the pixels in the scene. Once all WebWorkers (running the WASM module) are done with raytracing for their pixels, the resulting buffer is written to the screen by the main thread.

Some important files are described below.

* `client/` - The code running on the main thread
  * `index.ts` - The "main" file
  * `raytracer/` - The interfaces for the different raytracers
    * `singlecore.ts` - Spawns a single WASM instance on the main thread, with which it interacts
    * `multicore.ts` - Spawns 8 WebWorkers, with which it distributes the work and synchronises the results
* `shared/` - Library code that is shared (w.r.t. the compiler) between the main thread and the workers
* `worker/` - Code that is running on the webworker
  * `worker.ts` - The "main" file for the webworker. Mainly contains code to handle messages from the main thread

## 4. Credit
* [wasm-bindgen hello world example](https://github.com/rustwasm/wasm-bindgen/tree/master/examples/hello_world) - Setup for Rust to WASM with Webpack
* [Scratchapixel](https://www.scratchapixel.com/) - Nice explanation and code samples on raytracing
* [Torus ray intersection](http://cosinekitty.com/raytrace/chapter13_torus.html)
* [My old raytracer](https://github.com/dennis-school/raytrace_city/) - This I wrote during my BSc ([RuG - Computer Graphics](http://www.cs.rug.nl/svcg/Teaching/ComputerGraphics)), from which I reused some primitive intersection code (some of which I probably took from the course slides or provided code templates).
