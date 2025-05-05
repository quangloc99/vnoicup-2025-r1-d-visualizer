mod logic;

use bevy::{input::common_conditions::input_just_pressed, prelude::*, window::*};
use logic::board::{build_move_graph, Board};
use logic::cell::Cell;
use logic::solver;

use rand_chacha::rand_core::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;

const K: usize = 2;

const UNIT: f32 = 40.0;
const TARGET_RADIUS: f32 = 10.;
const TROMINO_BORDER_WIDTH: f32 = 3.0;
const CELL_PADDING: f32 = 3.0;
const BOARD_BORDER_WIDTH: f32 = 2.0;
const BOARD_CELL_SIZE: f32 = UNIT + 2. * CELL_PADDING + 2. * BOARD_BORDER_WIDTH;

const TROMINO_INNER_COLOR: Color = Color::WHITE;
const TROMINO_BORDER_COLOR: Color = Color::BLACK;
const BOARD_CELL_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);
const BOARD_BORDER_COLOR: Color = Color::WHITE;
const TARGET_COLOR: Color = Color::srgba(1., 0.2, 0.2, 0.7);

const RAISE_TIME: f32 = 0.2;
const ROTATE_TIME: f32 = 0.6;

const END_RAISE: f32 = RAISE_TIME;
const END_ROTATE: f32 = END_RAISE + ROTATE_TIME;
const END_LOWER: f32 = END_ROTATE + RAISE_TIME;

const RAISE_SCALE: f32 = 1.3;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.init_resource::<BoardState>();
    app.insert_resource(ClearColor(Color::srgb(0.8, 0.8, 0.8)));
    app.insert_resource::<AnimationData>(AnimationData {
        tromino_id: 0,
        org_quad: Cell(0, 0),
        target_quad: Cell(0, 0),
        start_time: 0.,
    });
    app.insert_state::<ActionState>(ActionState::PendingUpdate);
    app.insert_resource::<SolveState>(SolveState { org_cell: Cell(0, 0), target_cell: Cell(0, 0), move_count: 0 });
    app.insert_resource(RandomSource(ChaCha8Rng::seed_from_u64(177013)));

    app.add_systems(Startup, setup_text);
    app.add_systems(Startup, setup_target_cell);
    app.add_systems(Startup, setup_camera);
    app.add_systems(Startup, setup_board);
    app.add_systems(Startup, setup_grid);

    app.add_systems(Update, update_for_animation.run_if(in_state(ActionState::PendingUpdate)));
    app.add_systems(Update, animate_tronimo.run_if(in_state(ActionState::Animating)));
    app.add_systems(Update, mouse_to_select_target_cell.run_if(input_just_pressed(MouseButton::Left)));
    app.add_systems(Update, update_selected_cell.run_if(resource_changed::<SolveState>));
    app.add_systems(Update, update_text.run_if(resource_changed::<SolveState>));

    app.run();
}

#[derive(Component)]
struct LTromino {
    id: usize,
}

#[derive(Component)]
struct TargetCell;

#[derive(Component)]
struct TextCell;

#[derive(Component)]
struct TextMoveCount;

#[derive(Resource, Clone, Eq, PartialEq, Hash, Debug)]
struct BoardState {
    board: Board,
    move_graph: Vec<Vec<Cell>>,
    piece_coords: Vec<Cell>, // zero for the missing cell
}

#[derive(Resource, Clone, Eq, PartialEq, Debug)]
struct SolveState {
    org_cell: Cell,
    target_cell: Cell,
    move_count: usize,
}

#[derive(States, Clone, Eq, PartialEq, Hash, Debug)]
enum ActionState {
    PendingUpdate,
    Animating,
}

#[derive(Resource, Clone, PartialEq, Debug)]
struct AnimationData {
    tromino_id: usize,
    start_time: f32,
    org_quad: Cell,
    target_quad: Cell,
}

#[derive(Resource)]
struct RandomSource(ChaCha8Rng);

impl BoardState {
    fn new(k: usize) -> Self {
        let cur_missing = Cell(0, 0);
        let mut board = Board::new(k, cur_missing);
        let dim = board.limit().dim();
        let num_pieces = (dim - 1) / 3 + 1;
        let mut piece_coords = vec![Cell(std::usize::MAX, std::usize::MAX); num_pieces];
        for r in 0..board.limit().0 {
            for c in 0..board.limit().1 {
                let val = board[Cell(r, c)];
                piece_coords[val].0 = piece_coords[val].0.min(r);
                piece_coords[val].1 = piece_coords[val].1.min(c);
            }
        }
        let move_graph = build_move_graph(&mut board, cur_missing);

        Self { board, move_graph, piece_coords }
    }

    fn missing_corner_of(&self, piece_id: usize) -> Cell {
        assert!(piece_id > 0);
        let coor = self.piece_coords[piece_id];
        for r in 0..2 {
            for c in 0..2 {
                if self.board[coor + Cell(r, c)] != piece_id {
                    return Cell(r, c);
                }
            }
        }
        panic!("missing corner not found");
    }

    fn board_coor_shift(&self) -> Vec3 {
        let limit = self.board.limit();
        return -(BoardState::_tromino_coor(Cell(limit.0 - 1, limit.1 - 1)) / 2.);
    }
    fn _cell_coor(cell: Cell) -> Vec3 {
        let x = cell.1 as f32 * BOARD_CELL_SIZE;
        let y = -(cell.0 as f32) * BOARD_CELL_SIZE;
        Vec3::new(x, y, 0.)
    }

    fn _tromino_coor(cell: Cell) -> Vec3 {
        const SHIFT: f32 = BOARD_CELL_SIZE / 2.;
        BoardState::_cell_coor(cell) + Vec3::new(SHIFT, -SHIFT, 0.)
    }

    fn cell_coor(&self, cell: Cell) -> Vec3 {
        BoardState::_cell_coor(cell) + self.board_coor_shift()
    }

    fn tromino_coor(&self, cell: Cell) -> Vec3 {
        BoardState::_tromino_coor(cell) + self.board_coor_shift()
    }

    fn target_coor(&self, cell: Cell) -> Vec3 {
        self.cell_coor(cell).with_z(200.)
    }
}

impl Default for BoardState {
    fn default() -> Self {
        Self::new(K)
    }
}

fn missing_corner_to_angle(cell: Cell) -> f32 {
    match cell {
        Cell(0, 0) => 0.,
        Cell(1, 0) => std::f32::consts::FRAC_PI_2,
        Cell(1, 1) => std::f32::consts::PI,
        Cell(0, 1) => std::f32::consts::FRAC_PI_2 * 3.,
        _ => panic!("wrong corner"),
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    board_state: Res<BoardState>,
) {
    let inner_mesh = Mesh2d(meshes.add(build_l_tromino_mesh(UNIT, 0.)));
    let outer_mesh = Mesh2d(meshes.add(build_l_tromino_mesh(UNIT, TROMINO_BORDER_WIDTH)));
    let mat_inner = materials.add(TROMINO_INNER_COLOR);
    let mat_border = materials.add(TROMINO_BORDER_COLOR);

    let spawn_tromino = |cmd: &mut Commands, id: usize, coor: Vec3, angle: f32| {
        cmd.spawn((
            LTromino { id },
            Transform::from_translation(coor).with_rotation(Quat::from_rotation_z(angle)),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((inner_mesh.clone(), MeshMaterial2d(mat_inner.clone()), Transform::from_xyz(0., 0., 2.)));
            parent.spawn((outer_mesh.clone(), MeshMaterial2d(mat_border.clone()), Transform::from_xyz(0., 0., 1.)));
        });
    };
    for (&cell, i) in board_state.piece_coords[1..].iter().zip(1..) {
        let missing_corner = board_state.missing_corner_of(i);
        let angle = missing_corner_to_angle(missing_corner);
        spawn_tromino(&mut commands, i, board_state.tromino_coor(cell), angle);
    }
}

fn setup_target_cell(
    mut commands: Commands,
    board_state: Res<BoardState>,
    solve_state: Res<SolveState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let circle = Circle { radius: TARGET_RADIUS, ..default() };
    let mesh = meshes.add(circle.mesh().build());
    let material = materials.add(TARGET_COLOR);
    let cell = solve_state.target_cell;
    let coor = board_state.target_coor(cell);
    commands.spawn((
        TargetCell,
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_translation(coor),
        Visibility::default(),
    ));
}

fn setup_grid(mut commands: Commands, board_state: Res<BoardState>) {
    let limit = board_state.board.limit();
    for r in 0..limit.0 {
        for c in 0..limit.1 {
            let coor = board_state.cell_coor(Cell(r, c));
            commands
                .spawn((
                    Sprite {
                        color: BOARD_BORDER_COLOR,
                        custom_size: Some(Vec2::splat(BOARD_CELL_SIZE + BOARD_BORDER_WIDTH)),
                        ..default()
                    },
                    Transform::from_translation(coor.with_z(-100.)),
                ))
                .with_children(|parent| {
                    parent.spawn(Sprite {
                        color: BOARD_CELL_COLOR,
                        custom_size: Some(Vec2::splat(BOARD_CELL_SIZE - 2. * BOARD_BORDER_WIDTH)),
                        ..default()
                    });
                });
        }
    }
}

fn setup_text(mut commands: Commands, board_state: Res<BoardState>, solve_state: Res<SolveState>) {
    let coor = board_state.cell_coor(board_state.board.limit());
    let x = coor.x + 100.;
    commands.spawn((
        Text2d::new(format!(
            "({}, {}) -> ({}, {})",
            solve_state.org_cell.0, solve_state.org_cell.1, solve_state.target_cell.0, solve_state.target_cell.1
        )),
        TextColor(Color::BLACK),
        Transform::from_xyz(x, 20., 0.),
        TextCell,
    ));
    commands.spawn((
        Text2d::new(format!("{}", solve_state.move_count)),
        Transform::from_xyz(x, -20., 0.),
        TextColor(Color::BLACK),
        TextMoveCount,
    ));
}

fn build_l_tromino_mesh(unit: f32, border: f32) -> Mesh {
    let rec1 = Rectangle { half_size: Vec2::new(unit + border, unit / 2. + border) };
    let rec2 = Rectangle { half_size: Vec2::new(unit / 2. + border, unit + border) };
    let mut mesh1 = rec1.mesh().build();
    mesh1.translate_by(Vec3::new(0., -unit / 2., 0.));
    let mut mesh2 = rec2.mesh().build();
    mesh2.translate_by(Vec3::new(unit / 2., 0., 0.));
    mesh1.merge(&mesh2).ok();
    return mesh1;
}

fn update_for_animation(
    mut board_state: ResMut<BoardState>,
    mut solve_state: ResMut<SolveState>,
    mut animation_data: ResMut<AnimationData>,
    mut rng: ResMut<RandomSource>,
    timer: Res<Time>,
    cur_action_state: Res<State<ActionState>>,
    mut next_action_state: ResMut<NextState<ActionState>>,
) {
    if *cur_action_state != ActionState::PendingUpdate {
        return;
    }
    let missing_cell = board_state.piece_coords[0];
    if solve_state.target_cell == missing_cell {
        next_action_state.set(ActionState::PendingUpdate);
        return;
    }
    next_action_state.set(ActionState::Animating);
    solve_state.move_count += 1;

    let candidates = board_state.move_graph[missing_cell.encode(board_state.board.limit())]
        .iter()
        .map(|&x| (solver::dist(board_state.board.k(), solve_state.target_cell, x), x))
        .collect::<Vec<_>>();

    let min_dist = candidates.iter().map(|x| x.0).min().unwrap();
    let candidates = candidates.iter().filter(|x| x.0 == min_dist).map(|x| x.1).collect::<Vec<_>>();
    let candidate = candidates[rng.0.next_u64() as usize % candidates.len()];

    let tromino_id = board_state.board[candidate];
    let org_quad = board_state.missing_corner_of(tromino_id);
    board_state.board.make_move(missing_cell, candidate).unwrap();
    let target_quad = board_state.missing_corner_of(tromino_id);
    board_state.piece_coords[0] = candidate;

    *animation_data = AnimationData { tromino_id, start_time: timer.elapsed_secs(), org_quad, target_quad };
}

fn animate_tronimo(
    mut tromino_query: Query<(&mut Transform, &LTromino)>,
    animation_data: Res<AnimationData>,
    timer: Res<Time>,
    mut next_action_state: ResMut<NextState<ActionState>>,
) {
    let tromino_id = animation_data.tromino_id;
    let mut selected_entity = tromino_query.iter_mut().find(|(_, t)| t.id == tromino_id).map(|(t, _)| t).unwrap();
    selected_entity.translation.z = 100.0;

    let elapsed = timer.elapsed_secs() - animation_data.start_time;
    let mut org_angle = missing_corner_to_angle(animation_data.org_quad);
    let mut target_angle = missing_corner_to_angle(animation_data.target_quad);

    if animation_data.org_quad == Cell(0, 0) && animation_data.target_quad == Cell(0, 1) {
        org_angle = std::f32::consts::TAU;
    }
    if animation_data.org_quad == Cell(0, 1) && animation_data.target_quad == Cell(0, 0) {
        target_angle = std::f32::consts::TAU;
    }

    if elapsed < END_RAISE {
        let prog = EaseFunction::QuarticOut.sample(elapsed / RAISE_TIME).unwrap();
        let s = 1f32.lerp(RAISE_SCALE, prog);
        selected_entity.scale = Vec3::splat(s);
    } else if elapsed < END_ROTATE {
        let prog = EaseFunction::QuarticOut.sample((elapsed - END_RAISE) / ROTATE_TIME).unwrap();
        let angle = org_angle.lerp(target_angle, prog);
        selected_entity.rotation = Quat::from_rotation_z(angle);
    } else if elapsed < END_LOWER {
        selected_entity.rotation = Quat::from_rotation_z(target_angle);
        let prog = EaseFunction::QuarticOut.sample((elapsed - END_ROTATE) / RAISE_TIME).unwrap();
        let s = RAISE_SCALE.lerp(1.0, prog);
        selected_entity.scale = Vec3::splat(s);
    } else {
        selected_entity.scale = Vec3::splat(1.);
        selected_entity.translation.z = 0.;
        next_action_state.set(ActionState::PendingUpdate);
    }
}

fn mouse_to_select_target_cell(
    mut solve_state: ResMut<SolveState>,
    q_windows: Single<&Window, With<PrimaryWindow>>,
    q_camera: Single<(&Camera, &GlobalTransform)>,
    board_state: Res<BoardState>,
) {
    let (camera, camera_transform) = *q_camera;
    let Some(position) =
        q_windows.cursor_position().and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    else {
        return;
    };
    let top_left_coor = board_state.cell_coor(Cell(0, 0));
    let limit = board_state.board.limit();
    let r = ((top_left_coor.y - position.y) / BOARD_CELL_SIZE).round() as i32;
    let c = ((position.x - top_left_coor.x) / BOARD_CELL_SIZE).round() as i32;

    if r < 0 || c < 0 || r >= limit.0 as i32 || c >= limit.1 as i32 {
        return;
    }

    let cell = Cell(r as usize, c as usize);
    solve_state.org_cell = board_state.piece_coords[0];
    solve_state.target_cell = cell;
    solve_state.move_count = 0;
}

fn update_selected_cell(
    solve_state: Res<SolveState>,
    board_state: Res<BoardState>,
    mut target_query: Query<&mut Transform, With<TargetCell>>,
) {
    let coor = board_state.target_coor(solve_state.target_cell);
    let mut target_transform = target_query.single_mut().unwrap();
    target_transform.translation = coor;
}

fn update_text(
    solve_state: Res<SolveState>,
    mut q_text_move_count: Single<&mut Text2d, (With<TextMoveCount>, Without<TextCell>)>,
    mut q_text_cell: Single<&mut Text2d, (With<TextCell>, Without<TextMoveCount>)>,
) {
    q_text_cell.0 = format!(
        "({}, {}) -> ({}, {})",
        solve_state.org_cell.0, solve_state.org_cell.1, solve_state.target_cell.0, solve_state.target_cell.1
    );
    q_text_move_count.0 = format!("{}", solve_state.move_count);
}
