use crate::phonon_mesh;
use crate::phonon_mesh::instancing::StaticMeshes;
use bevy::prelude::*;
use bevy_fmod::libfmod::{Dsp, EventInstance};
use bevy_fmod::prelude::AudioListener;
use bevy_fmod::prelude::AudioSource;
use phonon::effects::direct::DirectApplyFlags;
use phonon::models::air_absorption::DefaultAirAbsorptionModel;
use phonon::models::directivity::Directivity;
use phonon::models::distance_attenuation::DefaultDistanceAttenuationModel;
use phonon::scene::coordinate_space::CoordinateSpace3f;
use phonon::simulators::direct::{DirectSimulator, DirectSoundPath, OcclusionType};
use phonon_fmod::parameter_spec::Params;
use std::os::raw::c_void;

#[derive(Component, Reflect)]
pub struct PhononSource {
    pub distance_attenuation: bool,
    pub air_absorption: bool,
    pub occlusion: bool,
    pub occlusion_type: OcclusionType,
    /// Size of the audio source when `OcclusionType` is set to `Volumetric`.
    pub occlusion_radius: f32,
    /// Number of occlusion samples to take when volumetric occlusion is enabled.
    /// Limited by `max_occlusion_samples` of the `DirectSimulator`.
    pub occlusion_samples: usize,
    // todo document what transmission is and what is needed to make it work (materials)
    pub transmission: bool,
    pub directivity: bool,
    pub hrtf_enable: bool,
}

impl Default for PhononSource {
    fn default() -> Self {
        PhononSource {
            distance_attenuation: true,
            air_absorption: true,
            occlusion: true,
            occlusion_type: OcclusionType::Volumetric,
            occlusion_radius: 1.0,
            occlusion_samples: 64,
            transmission: true,
            directivity: true,
            hrtf_enable: true,
        }
    }
}

#[derive(Resource)]
pub(crate) struct SteamSimulation {
    pub(crate) simulator: DirectSimulator,
    pub(crate) scene: phonon::scene::Scene,
}

pub struct PhononPlugin {
    /// Set this true to have `PhononPlugin` add a system which automatically
    /// adds a `PhononSource` to all FMOD audio sources. Note that the default
    /// settings of `PhononSource` may not fit your use case.
    pub auto_add_phonon_sources: bool,
    /// Sets the maximum number of occlusion samples, which is used when volumetric
    /// occlusion is enabled on a `PhononSource`.
    /// This only sets the max, the actual amount is set per source
    pub max_occlusion_samples: usize,
}

impl Default for PhononPlugin {
    fn default() -> Self {
        PhononPlugin {
            auto_add_phonon_sources: true,
            max_occlusion_samples: 512,
        }
    }
}

impl Plugin for PhononPlugin {
    fn build(&self, app: &mut App) {
        // This is the main scene to which all the geometry will be added later
        let scene = phonon::scene::Scene::new();

        let simulator = DirectSimulator::new(self.max_occlusion_samples);

        app.insert_resource(SteamSimulation { simulator, scene })
            .insert_resource(StaticMeshes::default())
            .register_type::<PhononSource>()
            .add_systems(
                Update,
                (
                    (
                        phonon_mesh::register_audio_meshes,
                        phonon_mesh::update_audio_mesh_transforms,
                    ),
                    update_steam_audio,
                    phonon_source_changed,
                )
                    .chain(),
            );

        if self.auto_add_phonon_sources {
            app.add_systems(Update, register_phonon_sources);
        }
    }
}

fn phonon_source_changed(query: Query<(&AudioSource, &PhononSource), Changed<PhononSource>>) {
    for (audio_source, component) in &query {
        if let Some(spatializer) = get_phonon_spatializer(audio_source.event_instance) {
            spatializer
                .set_parameter_bool(Params::DirectBinaural as i32, component.hrtf_enable)
                .unwrap()
        }
    }
}

fn update_steam_audio(
    mut sim_res: ResMut<SteamSimulation>,
    listener_query: Query<&GlobalTransform, With<AudioListener>>,
    audio_sources: Query<(&GlobalTransform, &AudioSource, &PhononSource)>,
) {
    // Commit changes to the sources, listener and scene.
    sim_res.scene.commit();

    let listener_transform = listener_query.get_single().unwrap();

    let listener_position = CoordinateSpace3f::from_vectors(
        listener_transform.forward().into(),
        listener_transform.up().into(),
        listener_transform.translation(),
    );

    for (source_transform, effect, settings) in audio_sources.iter() {
        // todo: Only search for the spatializer DSP if it hasn't been found before,
        // or if it's been moved
        if let Some(spatializer) = get_phonon_spatializer(effect.event_instance) {
            // todo reduce indentation
            let mut flags = DirectApplyFlags::empty();

            flags.set(
                DirectApplyFlags::DistanceAttenuation,
                settings.distance_attenuation,
            );
            flags.set(DirectApplyFlags::AirAbsorption, settings.air_absorption);
            flags.set(DirectApplyFlags::Occlusion, settings.occlusion);
            flags.set(DirectApplyFlags::Transmission, settings.transmission);
            flags.set(DirectApplyFlags::Directivity, settings.directivity);

            let source_position = CoordinateSpace3f::from_vectors(
                source_transform.forward().into(),
                source_transform.up().into(),
                source_transform.translation(),
            );

            let mut direct_sound_path = DirectSoundPath::default();

            let directivity = match settings.directivity {
                true => {
                    let valuestrlen = 0;
                    let (directivity_power, _) = spatializer
                        .get_parameter_float(Params::DirectivityDipolePower as i32, valuestrlen)
                        .unwrap();
                    let (directivity_weight, _) = spatializer
                        .get_parameter_float(Params::DirectivityDipoleWeight as i32, valuestrlen)
                        .unwrap();
                    Directivity {
                        dipole_weight: directivity_weight,
                        dipole_power: directivity_power,
                    }
                }
                false => Directivity::default(),
            };

            sim_res.simulator.simulate(
                &sim_res.scene,
                flags,
                &source_position,
                &listener_position,
                &DefaultDistanceAttenuationModel::default(),
                &DefaultAirAbsorptionModel::default(),
                directivity,
                settings.occlusion_type,
                settings.occlusion_radius,
                settings.occlusion_samples,
                1,
                &mut direct_sound_path,
            );

            let sound_path_ptr = &mut direct_sound_path as *mut _ as *mut c_void;
            let sound_path_size = size_of::<DirectSoundPath>();

            spatializer
                .set_parameter_data(
                    Params::DirectSoundPath as i32,
                    sound_path_ptr,
                    sound_path_size as u32,
                )
                .unwrap();
        }
    }
}

fn register_phonon_sources(
    mut audio_sources: Query<Entity, (Without<PhononSource>, With<AudioSource>)>,
    mut commands: Commands,
) {
    for audio_entity in audio_sources.iter_mut() {
        commands
            .entity(audio_entity)
            .insert(PhononSource::default());
    }
}

/// The goal here is to find the Steam Audio Spatializer DSP associated with an instance.
/// This way we can later set its parameters.
/// The DSP can basically be anywhere in the DSP chain, so we have to search for it.
fn get_phonon_spatializer(instance: EventInstance) -> Option<Dsp> {
    let dsp_description = "Phonon Spatializer";

    if let Ok(channel_group) = instance.get_channel_group() {
        let master_num_dsp = channel_group.get_num_ds_ps().unwrap();

        for index_dsp in 0..master_num_dsp {
            let dsp = channel_group.get_dsp(index_dsp).unwrap();
            let dsp_info = dsp.get_info().unwrap();

            if dsp_info.0 == dsp_description {
                return Some(dsp);
            }
        }

        // No spatializer DSP found in the master group, so continue
        // searching in the channel groups:
        let num_groups = channel_group.get_num_groups().unwrap();

        for index_group in 0..num_groups {
            let group = channel_group.get_group(index_group).unwrap();
            let group_num_dsp = group.get_num_ds_ps().unwrap();

            for index_dsp in 0..group_num_dsp {
                let dsp = group.get_dsp(index_dsp).unwrap();
                let dsp_info = dsp.get_info().unwrap();

                if dsp_info.0 == dsp_description {
                    return Some(dsp);
                }
            }
        }
    }

    None
}
