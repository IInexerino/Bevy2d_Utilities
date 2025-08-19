use bevy::{
    app::{
        App, 
        Plugin, 
        Startup, 
        Update
    }, 
    core_pipeline::core_2d::Camera2d, 
    ecs::{
        event::EventReader, query::With, resource::Resource, schedule::IntoScheduleConfigs, system::{
            Commands, 
            Res, 
            Single, 
        }
    }, input::{
        keyboard::KeyCode, 
        mouse::MouseWheel, 
        ButtonInput
    }, math::{
        Vec2, 
        Vec3
    }, 
    render::camera::Projection, 
    transform::components::Transform 
};

/// A [`Plugin`] that defines an interface for camera dynamicity support in Bevy
#[derive(Clone)]
pub struct Dynamic2dCameraPlugin {
    /// Settings for spawning the camera.
    /// 
    /// `Some(custom_camera)` will spawn an entity with `custom_camera` as a component.
    /// `None` will not spawn a `Camera2d`.
    /// 
    /// Defaults to `Some(Camera2d::default())`.
    pub spawn_camera: Option<Camera2d>,

    /// Whether to enable wasd camera movment or not, wheather to restrict it xxyy or not
    /// 
    /// `None` will result in not registering any [`build_wasd_move_camera_system`] system
    /// or adding a [`CameraMoveConfigs`] resource to the world
    /// 
    /// Any variation of `Some(_)` will use [`build_wasd_move_camera_system`] to construct 
    /// a closure system, will add its [`CameraMoveConfigs`] to the world as a [`Resource`], 
    /// and add the created closure system to the [`Update`] schedule with the rc field of 
    /// the [`CameraMoveConfigs`] resource being related to its run condition
    /// 
    /// `Some(CameraMoveConfigs::new(true / false, movement_speed, None))` will construct 
    /// a closure system with the given speed
    /// 
    /// `Some(CameraMoveConfigs::new(true / false, movement_speed, Some((x, -x, y, -y))))` 
    /// will construct a closure system with the given speed and translation restrictions
    pub enable_wasd_movment: Option<CameraMoveConfigs>,

    /// Whether to enable camera scroll zooming or not, 
    /// wheather or not to restrict its upper bounds, lower bounds, or both.
    /// 
    /// `None` will result in not registering any [`build_scroll_zoom_camera_system`] system
    /// or adding a [`CameraZoomConfigs`] resource to the world
    /// 
    /// Any variation of `Some(_)` will use [`build_scroll_zoom_camera_system`] to construct 
    /// a closure system, will add its [`CameraZoomConfigs`] to the world as a [`Resource`], 
    /// and add the created closure system to the [`Update`] schedule with the rc field of 
    /// the [`CameraZoomConfigs`] resource being related to its run condition
    /// 
    /// `Some(CameraZoomConfigs::new(true / false, None, None, movement_speed))` 
    /// will construct a closure system with the given speed
    /// 
    /// Any variation of `Some(CameraZoomConfigs::new(true / false, Some(lower_limit), None, movement_speed))` 
    /// will construct a closure system with the given speed, as well as the upper and/or lower bounds
    pub enable_scroll_zoom: Option<CameraZoomConfigs>
}

impl Default for Dynamic2dCameraPlugin {
    fn default() -> Self {
        Dynamic2dCameraPlugin{ 
            spawn_camera: Some(Camera2d::default()),
            enable_wasd_movment: None,
            enable_scroll_zoom: None
        }
    }
}

impl Plugin for Dynamic2dCameraPlugin {
    fn build(&self, app: &mut App) {
        if let Some(chosen_2dcamera) = self.spawn_camera.clone() {
            app.add_systems(Startup,  build_spawn_camera_system(chosen_2dcamera));
        }
        if let Some(camera_move_configs) = self.enable_wasd_movment.clone() {
            app.insert_resource(camera_move_configs.clone());
            app.add_systems(Update, build_wasd_move_camera_system(camera_move_configs).run_if(run_if_configured_to_move));
        }
        if let Some(camera_zoom_configs) = self.enable_scroll_zoom.clone() {
            app.insert_resource(camera_zoom_configs.clone());
            app.add_systems(Update, build_scroll_zoom_camera_system(camera_zoom_configs).run_if(run_if_configured_to_zoom));
        }
    }
}

/// Run condition which checks whether the configuration as a resource says to run or not to run a system
fn run_if_configured_to_move(camera_movement_configs: Res<CameraMoveConfigs>) -> bool { camera_movement_configs.rc }

/// Run condition which checks whether the configuration as a resource says to run or not to run a system
fn run_if_configured_to_zoom(camera_movement_configs: Res<CameraZoomConfigs>) -> bool { camera_movement_configs.rc }

/// Build closure which spawns a custom `Camera2d`
pub fn build_spawn_camera_system(camera2d: Camera2d) -> impl FnMut(Commands) {
    move | mut commands: Commands | {
        commands.spawn(camera2d.clone());
    }
}

/// Configurations for camera movement speed, and optional configurations for (right, left, top, bottom) movement limits 
#[derive(Clone, Resource)]
pub struct CameraMoveConfigs {
    /// Run condition to the closure system configured by this
    pub rc: bool,
    
    /// The speed will be multiplied by a normalized `Vec2`, and added to `transform.translation` if unobstructed
    pub speed: f32,

    /// `Some((f32,f32,f32,f32))` will add correspondingly: (right, left, top, bottom) movemement limits, which will set movement into the direction in question to 0`
    /// 
    /// This field is optional. `None` will result in no limits
    pub xxyy_limits: Option<(f32,f32,f32,f32)>,
}

impl CameraMoveConfigs {
    pub fn new(rc: bool, speed: f32, xxyy_limits: Option<(f32,f32,f32,f32)>) -> Self {
        CameraMoveConfigs {
            rc,
            speed,
            xxyy_limits
        }
    }
}

/// Build a closure which takes in custom [`CameraMoveConfigs`], checks WASD input, 
/// and changes the `Transform.translation` of the [`Entity`] with the [`Camera2d`] 
/// component accordingly - in order to move the camera [`Entity`].
pub fn build_wasd_move_camera_system(camera_movement_configs: CameraMoveConfigs) -> impl FnMut(
    Single<&mut Transform, With<Camera2d>>,
    Res<ButtonInput<KeyCode>>
) {
    move | 
        query_camera: Single<&mut Transform, With<Camera2d>>, 
        keys: Res<ButtonInput<KeyCode>>
    |{
        let mut movement = Vec2::new(0.,0.);

        if keys.pressed(KeyCode::KeyW) || keys.just_pressed(KeyCode::KeyW) {
            movement.y += 1.;
        }
        if keys.pressed(KeyCode::KeyS) || keys.just_pressed(KeyCode::KeyS) {
            movement.y += -1.;
        }
        if keys.pressed(KeyCode::KeyD) || keys.just_pressed(KeyCode::KeyD) {
            movement.x += 1.;
        }
        if keys.pressed(KeyCode::KeyA) || keys.just_pressed(KeyCode::KeyA){
            movement.x += -1.;
        }

        if movement != Vec2::new(0., 0.) {
            movement = movement.normalize();

            let mut movement = Vec3::new(
                movement.x * camera_movement_configs.speed, 
                movement.y * camera_movement_configs.speed, 
                0.0_f32
            );

            let mut transform = query_camera.into_inner();

            // BUG, doesnt work for some reason, for the moment set to none
            if let Some((right_x, left_x, up_y, down_y)) = camera_movement_configs.xxyy_limits {
                if movement.x + transform.translation.x >= right_x || movement.x + transform.translation.x <= left_x  {
                    movement.x = 0.
                } 
                if movement.y + transform.translation.y >= up_y || movement.y + transform.translation.y <= down_y  {
                    movement.y = 0.
                } 
            }

            transform.translation += movement;
        }
    }
}

#[derive(Clone, Resource)]
pub struct CameraZoomConfigs {
    /// Run condition to the closure system configured by this
    pub rc: bool,

    /// The speed will be multiplied by a normalized `Vec2`, and added to `transform.translation` if unobstructed
    pub speed: f32,
   
    /// Sets lower limit to changes of `OrthographicProjection.scale` in system built from [`build_scroll_zoom_camera_system`] 
    pub limit_min: Option<f32>,
    
    /// Sets upper limit to changes of `OrthographicProjection.scale` in system built from [`build_scroll_zoom_camera_system`] 
    pub limit_max: Option<f32>,
}

impl CameraZoomConfigs {
    pub fn new(rc: bool, speed: f32, limit_min: Option<f32>, limit_max: Option<f32>) -> Self {
        CameraZoomConfigs {
            rc,
            speed,
            limit_min,
            limit_max,
        }
    }
}

/// Build a closure which takes in custom [`CameraZoomConfigs`], checks mouse_scroll 
/// input through related events, and changes the `OrthographicProjection.scale` of 
/// the [`Entity`] with the [`Projection`] component accordingly - in order to change 
/// the projection scale of the camera [`Entity`].
pub fn build_scroll_zoom_camera_system(camera_zoom_configs: CameraZoomConfigs) -> impl FnMut(
    EventReader<MouseWheel>,
    Single<&mut Projection, With<Camera2d>>,
) {
    move |
        mut evr_scroll: EventReader<MouseWheel>,
        mut query_camera: Single<&mut Projection, With<Camera2d>>
    | {
        if let Some(mouse_wheel) = evr_scroll.read().next() {
            match query_camera.as_mut() {
                Projection::Orthographic(ortho) => {
                    // Alter the zoom
                    println!("Attempting to alter the zoom\nScale = {}\nScroll = x:{} y:{}", ortho.scale, mouse_wheel.x, mouse_wheel.y);
                    let new_ortho_scale = ortho.scale + -(mouse_wheel.y * camera_zoom_configs.speed);

                    if let Some(min) = camera_zoom_configs.limit_min {
                        if let Some(max) = camera_zoom_configs.limit_max {
                            if new_ortho_scale >= min && new_ortho_scale <= max {
                                ortho.scale = new_ortho_scale;
                            }
                        } else {
                            if new_ortho_scale >= min {
                                ortho.scale = new_ortho_scale;
                            }
                        }
                    } else {
                        if let Some(max) = camera_zoom_configs.limit_max {
                            if new_ortho_scale <= max {
                                ortho.scale = new_ortho_scale;
                            }
                        } else {
                            ortho.scale = new_ortho_scale;
                        }
                    }
                }
                _ => {
                    eprintln!("Scrolling Error: Projection is not Orthograpic as should be by Default");
                }
            }
        }
    }
}