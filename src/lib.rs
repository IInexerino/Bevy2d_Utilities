pub mod dynamic_camera;
pub mod window;
pub mod grids;

pub mod prelude {
    #[doc(hidden)]
    pub use crate::dynamic_camera::{
        CameraMoveConfigs,
        CameraZoomConfigs
    };
}

use bevy::app::plugin_group;


plugin_group! {
    pub struct Bevy2dUtilitiesPlugin {
        dynamic_camera:::Dynamic2dCameraPlugin,
        window:::WindowUtilPlugin
    }
}