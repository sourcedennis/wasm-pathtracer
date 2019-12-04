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
TODO

## 4. Credit
* [wasm-bindgen hello world example](https://github.com/rustwasm/wasm-bindgen/tree/master/examples/hello_world) - Setup for Rust to WASM with Webpack
* [Scratchapixel](https://www.scratchapixel.com/) - Nice explanation and code samples on raytracing
* [Torus ray intersection](http://cosinekitty.com/raytrace/chapter13_torus.html)
* [My old raytracer](https://github.com/dennis-school/raytrace_city/) - This I wrote during my BSc ([RuG - Computer Graphics](http://www.cs.rug.nl/svcg/Teaching/ComputerGraphics)), from which I reused some primitive intersection code (some of which I probably took from the course slides or provided code templates).
