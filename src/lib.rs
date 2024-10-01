//! phonon_rs integration for Bevy.

mod phonon_mesh;
mod phonon_plugin;

pub mod prelude {
    pub use crate::phonon_mesh::materials;
    pub use crate::phonon_mesh::NeedsAudioMesh;
    pub use crate::phonon_plugin::PhononPlugin;
    pub use crate::phonon_plugin::PhononSource;
}
