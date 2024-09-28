# Bevy Phonon

A bevy integration for the unofficial Steam Audio Rust port, [phonon_rs].

As phonon_rs is game engine and audio engine independent,
a choice must be made for the audio engine.
Currently only FMOD is supported ([bevy_fmod] specifically). Kira support is possible,
pending the following [issue](https://github.com/NiklasEi/bevy_kira_audio/issues/127).

## Features

- Panning Effect (stereo only for now)
- Direct Effect
    - Distance attenuation
    - Air absorption
    - Occlusion
    - Transmission (one material per mesh for now)
    - Directivity

Planned: HRTF, geometry-based reverb and more.
todo add picture of Phonon spatializer

## Usage

1. First follow the setup of [bevy_fmod].
2. Download the phonon_rs FMOD plugin from -todo- or build from source
using `cargo build -p phonon-fmod --release`.
3. Place the FMOD plugin into one of the [FMOD plugin directories].
4. When building from source pay extra attention to also copy phonon_fmod.plugin.js to the plugin directory.
5. In FMOD Studio you can now add the Phonon Spatializer effect to your event tracks.
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