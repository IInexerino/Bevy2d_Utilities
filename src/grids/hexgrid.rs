use bevy::{prelude::*};

use std::{
    collections::HashSet, 
    sync::atomic::{
        AtomicU64, 
        Ordering
    }
};

/// A placeholder [`Component`] for a hexagonal tile entity that keeps track of its grid coordinates.
/// 
/// This component is attached as a child of the [`Entity`] containing the [`HexGrid`] component.
/// 
/// The `x` and `y` fields denote its hexgrid coordinates.
#[derive(Component, Clone)]
pub struct HexTile {
    pub x: u32,
    pub y: u32,
}

impl HexTile {
    pub fn new(x: u32, y: u32) -> Self {
        HexTile{ x, y, }
    }

    /// Returns the truncated intended `translation` of the [`Entity`] containing this [`HexTile`] 
    /// relative to the center-point of the [`HexGrid`] containing [`Entity`] of which it is a child
    /// by using data obtained describing the [`HexGrid`] in question.
    pub fn coord_to_world(
        &self, 
        hextile_width: f32, 
        columns: u32, 
        rows: u32,
        orientation: HexGridOrientation,
    ) -> Vec2 {
        let hextile_height = hextile_width * 0.866;

        match orientation {
            HexGridOrientation::Vertical => {
                let x = (((self.x as f32) * hextile_width * 0.75) - ((hextile_width * 0.75) * (((columns / 2) as f32)) - hextile_width * 0.375)) - (if columns % 2 != 0 {
                    hextile_width * 0.375
                } else { 0.0 });
                let y = ((self.y as f32) * hextile_height + (if self.x % 2 != 0 { 
                    hextile_height / 2.0 
                } else { 0.0 })) - (hextile_height * ((rows / 2) as f32) - (hextile_height / 4.0)) - (if rows % 2 != 0 {
                    hextile_height / 2.0
                } else { 0.0 });
                
                return Vec2::new(x,y);
            },
            HexGridOrientation::Horizontal => {
                println!("Error: not functional yet, fucking wait");
                panic!()
            }
        }
    }

    /// Returns the order of this [`HexTile`] within the [`HexGrid`] if we were to start at 
    /// `(col: 0, row: 0) -> 1`, increment by each tile moved through the columns, and 
    /// increment each time we reach a new row.
    /// 
    /// e.g. if `columns = 5` then `(col: 0, row: 1) -> 6`.
    pub fn coord_to_order(
        &self,
        columns: u32, 
    ) -> u32 {
        (self.x + 1) + (self.y * columns)
    }

    /// Returns a [`Vec<(u32, u32)>`] containing a list of this [`HexTile`]'s 
    /// neighbour hextiles' `x` and `y` hexgrid coordinates.
    /// 
    /// Takes into account [`HexGrid`] size and bounds, and returns only 
    /// the coords of existing neighbors.
    pub fn get_neighbors(
    &self, 
    columns: u32, 
    rows: u32,
    orientation: HexGridOrientation,
    ) -> Vec<(u32, u32)> {
        let mut neighbors = Vec::new();
        let x = self.x as i32;
        let y = self.y as i32;

        match orientation {
            HexGridOrientation::Vertical => {
                // offsets for odd/even columns
                let offsets: &[(i32, i32)] = if x % 2 == 0 {
                    // even column
                    &[ (1, 0), (-1, 0), (0, -1), (0, 1), (1, -1), (-1, -1) ]
                } else {
                    // odd column
                    &[ (1, 0), (-1, 0), (0, -1), (0, 1), (1, 1), (-1, 1) ]
                };

                for (dx, dy) in offsets {
                    let nx = x + dx;
                    let ny = y + dy;
                    if nx >= 0 && ny >= 0 && nx < columns as i32 && ny < rows as i32 {
                        neighbors.push((nx as u32, ny as u32));
                    }
                }
            }
            HexGridOrientation::Horizontal => {
                println!("Error: not functional yet, fucking wait");
                panic!();
            }
        }
        neighbors
    }

    /// Returns a [`Bundle`] of components containing the [`HexTile`], a [`Transform`] 
    /// with a translation corresponding to its relative position to the center of its 
    /// [`HexGrid`] containing parent [`Entity`], and a [`Visibility::Visible`].
    /// 
    /// Does not contain [`Sprite`].
    /// 
    /// Consumes the [`HexTile`].
    pub fn get_bundle(
        self,
        orientation: HexGridOrientation,
        columns: u32, 
        rows: u32, 
        hextile_width: f32
    ) -> impl Bundle {
        
        let relative_pos = self.coord_to_world(hextile_width, columns, rows, orientation);

        match orientation {
            HexGridOrientation::Vertical => {
                return 
                (
                    self,
                    Transform::from_xyz(relative_pos.x, relative_pos.y, 0.),
                    Visibility::Visible,
                );
            },
            HexGridOrientation::Horizontal => {
                println!("Error: not functional yet, fucking wait");
                panic!()
            }
        }
    }
}

// Global counter, starts at 1 because fetch_add returns the previous value
/// Counter for the amount of [`HexGrid`] instances that have been created with [`HexGrid::new()`].
static HEXGRID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Resets the atomic integer counter to 1.
pub fn reset_hexgrid_counter() { HEXGRID_COUNTER.store(1, Ordering::Relaxed); }

/// Defines the orientation of the HexCells (pointy to the side, or upwards).
#[derive(Clone, Copy)]
pub enum HexGridOrientation {
    /// Straight columns offset by 0.75.
    /// 
    /// Horizontal part of the hexagon is upwards and downwards.
    Vertical,

    /// Straight rows offset by 0.75.
    /// 
    /// Horizontal part of the hexagon is to the left and right.
    Horizontal
}

/// A [`Component`] for a grid of hexagonal cells [`Entity`] containing an 
/// incremental id, as well as data which describes the grid.
/// 
/// This component is a parent to the [`HexTile`] entities which it contains.
#[derive(Component, Clone)]
pub struct HexGrid{
    /// A unique identifier counter which begins at one, and increments for 
    /// each new instance of [`HexGrid`] created with [`HexGrid::new()`].
    pub id: u64,

    /// The orientation of its hexagon tiles.
    /// 
    /// `vertical = corners to the sides`.
    /// 
    /// `horizontal = corners to the top and bottom`.
    pub orientation: HexGridOrientation,

    /// Number of columns in the grid (x).
    pub columns: u32,

    /// Number of rows in the grid (y).
    pub rows: u32,

    /// Pixel width of the individual hextiles.
    pub hextile_width: f32,
}

impl HexGrid {
    /// Returns a [`HexGrid`] with an `id` larger by one then the last instance created by this method (unless reset with [`reset_hexgrid_counter()`]).
    pub fn new(
        orientation: HexGridOrientation, 
        columns: u32, 
        rows: u32, 
        hextile_width: f32
    ) -> Self {

        let id = HEXGRID_COUNTER.fetch_add(1, Ordering::Relaxed);

        HexGrid {
            id,
            orientation,
            columns,
            rows,
            hextile_width
        }
    }

    /// Builds an 'exclusive' [`System`] closure which spawns an [`Entity`] using 
    /// config data of a particular instance of [`HexGrid`] at specified
    /// translation coordinates relative to the world (global).
    /// 
    /// The [`HexTile`] containing children entities of this [`Entity`] will not 
    /// be immediately spawned with a [`Sprite`] component.
    /// 
    /// Can be added to the [`Startup`] schedule, also can be used as a one-shot system.
    /// 
    /// Theoretically works just as well for the construction of horizontal hexgrids {untested!}.
    pub fn build_spawn_hexgrid_entity_system( self , hexgrid_translation: Vec3) -> impl FnMut( &mut World ) {

        move |    
            world: &mut World
        | {
            // Spawn the HexGrid containing Entity with the specified translation and get its id
            let parent_grid = world.spawn((
                self.clone(),
                // `GlobalTransform` required & children's `Transform`s are relative to this rather than to world coordinates
                Transform::from_translation(hexgrid_translation),
                Visibility::Hidden,
            ))
            .id();

            // Spawn (columns * rows) * HexTile containing entities
            world.entity_mut(parent_grid).with_children(|parent_builder| {
                for col in 0..self.columns {
                    for row in 0..self.rows {

                        // Spawn the HexTile entities as children of the HexGrid
                        parent_builder.spawn(
                            HexTile::new(col,row).get_bundle(
                                self.orientation,
                                self.columns,
                                self.rows,
                                self.hextile_width
                            )
                        );
                    }
                }
            });
        }
    }
}

/// Allows either a single path, or multiple paths towards images to be used for textures.
#[derive(Clone)]
pub enum TileTextures {
    /// Must contain path towards desired image
    Single(String),

    /// In `.1` it contains the default texture path, to which all textures unspecified in `.0` will be set
    /// 
    /// `.0` contains a vector with a collection of tuples in which `.1` is a 
    /// path towards a texture image, and `.0` is a vector with a collection of 
    /// ranges in hexgrid tile order notation (see [`HexTile::coord_to_order()`])
    Multiple(Vec<(Vec<(u32, u32)>, String)>, String)
}

/// Builds a [`System`] closure which inserts or rewrites a [`Sprite`] component configured
/// in various ways by [`TileTextures`] to all children entities of the 
/// [`HexGrid`] specified by it's id.
/// 
/// Primarily used for loading levels, as all previous [`Sprite`] configurations 
/// of all of the children entities of the specified [`HexGrid`] will be rewritten in all cases. 
/// 
/// (If you need to change the texture of a particular [`HexTile`] entity, use
/// [`build_change_hextile_textures_system()`]).
/// 
/// Can be added to the [`Startup`] schedule if set to run after the entities have 
/// been spawned, also can be used as a one-shot system.
/// 
/// Theoretically works just as well for the construction of horizontal hexgrids {untested!}.
pub fn build_change_hexgrid_textures_system(
    textures_configs: TileTextures,
    grid_id: u64,
) -> impl FnMut(
    Commands,
    Res<AssetServer>,
    Query<(&Children, &HexGrid)>,
    Query<&HexTile>
) {

    let textures_configs = textures_configs.clone();

    move |
        mut commands: Commands,
        asset_server: Res<AssetServer>, 
        children_query: Query<(&Children, &HexGrid)>,
        hextile_query: Query<&HexTile>
    | {
        for (children, hexgrid) in &children_query {

            if hexgrid.id != grid_id {
                continue;
            }

            let mut texture = match &textures_configs {
                TileTextures::Single(path) => asset_server.load(path),
                TileTextures::Multiple( _ , default_path ) => asset_server.load(default_path)
            };

            for &child in children {
                if let Ok(hextile) = hextile_query.get(child){
                
                    let order_pos = hextile.coord_to_order(hexgrid.columns);

                    if let TileTextures::Multiple(ranges_and_paths, _ ) = &textures_configs {

                        if let Some((_, path)) = ranges_and_paths
                        .iter()
                        .find(|(ranges, _ )| ranges
                            .iter()
                            .any(|&(start, end)| order_pos >= start && order_pos <= end)) {
                            texture = asset_server.load(path);
                        }


                        for (ranges, path) in ranges_and_paths {
                            for range in ranges {
                                if order_pos >= range.0 && order_pos <= range.1 {
                                    texture = asset_server.load(path); 
                                }
                            }
                        }
                    }

                    commands.entity(child).insert((
                        Sprite {
                            custom_size: Some(Vec2::new(
                                hexgrid.hextile_width, 
                                hexgrid.hextile_width * 0.866
                            )),
                            image: texture.clone(),
                            ..Default::default()
                        },
                    ));
                }
            }
        } 
    }
}

/// Builds a [`System`] closure which inserts or rewrites a [`Sprite`] component configured
/// by the texture path, to children entities of the [`HexGrid`] specified by id whose 
/// [`HexTile`] coordinates correspond to the specified hextiles_coords.
/// 
/// Used for changing the spites of specieif [`HexTile`] containing entities. Only tiles at the specified coordinates will be altered 
/// 
/// (If you need to change the texture of all [`HexTile`] entities that are children of a given [`HexGrid`] entity, use
/// [`build_change_hexgrid_textures_system()`]).
/// 
/// Can be added to the [`Startup`] schedule, also can be used as a one-shot system.
/// 
/// Theoretically works just as well for the construction of horizontal hexgrids {untested!}.
pub fn build_change_hextile_textures_system(
    grid_id: u64,
    texture_path: &str,
    hextiles_coords: HashSet<(u32,u32)>
) -> impl FnMut(
    Commands,
    Res<AssetServer>, 
    Query<(&Children, &HexGrid)>,
    Query<&HexTile>,
) {
    let texture_path = texture_path.to_string();

    move |
        mut commands: Commands,
        asset_server: Res<AssetServer>, 
        children_query: Query<(&Children, &HexGrid)>,
        hextile_query: Query<&HexTile>
    | {
        let texture =  asset_server.load(&texture_path);

        for (children, hexgrid) in &children_query {

            if hexgrid.id != grid_id {
                continue;
            }

            for &child in children {
                if let Ok(hextile) = hextile_query.get(child) {
                    if hextiles_coords.contains(&(hextile.x, hextile.y)) {
                        commands.entity(child).insert((
                            Sprite {
                                custom_size: Some(Vec2::new(hexgrid.hextile_width, hexgrid.hextile_width * 0.866)),
                                image: texture.clone(),
                                ..Default::default()
                            },
                        ));
                    }
                }
            }
        }
    }
}