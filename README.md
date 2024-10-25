# Bevy Phonon

A bevy integration for the unofficial Steam Audio Rust port, [phonon_rs].  
Demo: https://youtu.be/DFIYmiytqAw TODO: Update demo with HRTF and transmission.

As phonon_rs is game engine and audio engine independent,
a choice must be made for the audio engine.
Currently only FMOD is supported (through [bevy_fmod] specifically). Kira support is possible,
pending the following [issue](https://github.com/NiklasEi/bevy_kira_audio/issues/127).

⚠️ Warning: I don't know yet how this crate will fare in real projects (especially compared to Steam Audio).
If you have any data/issues in this regard, please consider contributing or contacting me 
through Discord (user: ixml). If it's working well for you I'd also like to hear that!

## Features

- Panning Effect (stereo only for now)
- Binaural Effect (built-in HRTF only for now)
- Direct Effect
    - Distance attenuation
    - Air absorption
    - Occlusion
    - Transmission (one material per mesh for now)
    - Directivity

TODO: Update with latest addition (hrtf)
![FMOD Phonon Spatializer](/media/phonon-spatializer.png)

Planned: Geometry-based reverb (CPU/GPU), baking and more.

## Usage

1. First follow the setup of [bevy_fmod].
2. Clone https://github.com/GitGhillie/phonon_rs and build the FMOD plugin
using `cargo build -p phonon-fmod --release`.
3. Place the FMOD plugin into one of the [FMOD plugin directories].
4. Copy phonon_fmod.plugin.js to the plugin directory. TODO add link.
5. In FMOD Studio you can now add the Phonon Spatializer effect to your event tracks:
   ![FMOD Plugin Selection](/media/plugin-selection.png)
6. On the Bevy side update FmodPlugin to include the path to the FMOD plugin, and
add the PhononPlugin:
```rust
    .add_plugins((
        DefaultPlugins,
        FmodPlugin {
            audio_banks_paths: &[
                "./assets/fmod-project/Build/Desktop/Master.bank",
                "./assets/fmod-project/Build/Desktop/Master.strings.bank",
                "./assets/fmod-project/Build/Desktop/Music.bank",
            ],
            plugin_paths: Some(&["./assets/fmod-project/Plugins/libphonon_fmod.so"]),
        },
    ))
    .add_plugins(PhononPlugin::default())
```
- See https://github.com/Salzian/bevy_fmod/blob/main/examples/spatial.rs for setting up FMOD sources and listeners.
By default, FMOD sources will automatically get a `PhononSource` component for simulation.
See `PhononPlugin` documentation if you want to change this.

## License

Licensed under Apache-2.0

[phonon_rs]: https://github.com/GitGhillie/phonon_rs
[bevy_fmod]: https://crates.io/crates/bevy_fmod
[FMOD plugin directories]: https://www.fmod.com/docs/2.02/studio/plugin-reference.html#loading-plug-ins