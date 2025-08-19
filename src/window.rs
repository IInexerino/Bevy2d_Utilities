use bevy::{ 
    app::{
        App, 
        Plugin, 
        Update,
    }, ecs::{
        resource::Resource, 
        system::{
            Query, 
            Res
        }
    }, 
    input::{
        keyboard::KeyCode, 
        ButtonInput
    }, 
    window::{
        MonitorSelection, 
        VideoModeSelection, 
        Window, 
        WindowMode
    }
};

/// A [`Plugin`] that defines an interface for common window functionality support in Bevy
#[derive(Clone, Default)]
pub struct WindowUtilPlugin;

// Interesting question would be - how to make setting choices persist into the next time of opening the app
impl Plugin for WindowUtilPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WindowConfigs::new((512,512), FullScreenConfig::Fullscreen));

        app.add_systems(Update, f11_change_window_mode);
    }
}

pub enum FullScreenConfig {
    Fullscreen,
    BorderlessFullscreen
}

#[derive(Resource)]
struct WindowConfigs {
    size: (u32, u32),
    full_screen_mode: FullScreenConfig
}

impl WindowConfigs {
    fn new(size: (u32, u32), full_screen_mode: FullScreenConfig) -> Self {
        WindowConfigs { 
            size, 
            full_screen_mode
        }
    }
}

/// 
fn f11_change_window_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    window_configs: Res<WindowConfigs>,
    mut windows: Query<&mut Window>,
) {
    if keyboard.just_pressed(KeyCode::F11) {

        for mut window in  windows.iter_mut() {
            match window_configs.full_screen_mode {
                FullScreenConfig::Fullscreen => {
                    window.mode = match window.mode {
                        WindowMode::Windowed => WindowMode::Fullscreen(MonitorSelection::Current, VideoModeSelection::Current),
                        _ => WindowMode::Windowed,
                    };
                },
                FullScreenConfig::BorderlessFullscreen => {
                    window.mode = match window.mode {
                        WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                        _ => WindowMode::Windowed,
                    
                    };
                }
            }
        }
    }
}