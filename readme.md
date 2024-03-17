# Config-driven UI POC
A proof of concept of generating a UI on top of egui based on configuration. This requires adding a layer that defines the UI components to render and the yaml config to use.

This code re-produces the [hello_world example egui](https://github.com/emilk/egui/blob/master/examples/hello_world/src/main.rs) into a [config](./config/app.yaml) file and a set of Context/Handlers/Readers

## Benefits
- Change the UI dynamically without re-compiling
  - Allows faster iteration for practical development
  - Easier Hot-reloading
- Separate and re-use code more easily
