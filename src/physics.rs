use rapier2d::{math::Real, na::{Isometry2, Vector2}, prelude::{BroadPhase, CCDSolver, ColliderBuilder, ColliderHandle, ColliderSet, IntegrationParameters, IslandManager, JointSet, NarrowPhase, PhysicsPipeline, RigidBody, RigidBodyBuilder, RigidBodyHandle, RigidBodySet}};
use tetra::{State, graphics::{Color, DrawParams}, math::Vec2};

pub type V2 = Vec2<f32>;

pub struct PhysicsHandle(pub RigidBodyHandle, pub ColliderHandle);

pub struct Physics {
    pub rb_set: RigidBodySet,
    pub coll_set: ColliderSet,
    integration_params: IntegrationParameters,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    joint_set: JointSet,
    ccd_solver: CCDSolver,
    physics_pipeline: PhysicsPipeline
}

impl Physics {
    pub fn setup() -> Physics {
        Physics {
            rb_set: RigidBodySet::new(),
            coll_set: ColliderSet::new(),
            integration_params: IntegrationParameters::default(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            joint_set: JointSet::new(),
            ccd_solver: CCDSolver::new(),
            physics_pipeline: PhysicsPipeline::new()
        }
    }

    pub fn build_ship_collider(&mut self, half_x: f32, half_y: f32) -> PhysicsHandle {
        let rb = RigidBodyBuilder::new_dynamic()
            .linear_damping(1.5).angular_damping(2.5).build();
        let rb_handle = self.rb_set.insert(rb);
        let coll = ColliderBuilder::cuboid(half_x * 0.85, half_y * 0.85)
            .density(1.0).friction(2.0).restitution(0.1).build();
        let coll_handle = self.coll_set.insert_with_parent(coll, rb_handle,
            &mut self.rb_set);
        PhysicsHandle(rb_handle, coll_handle)
    }

    pub fn build_island_collider(&mut self, half_x: f32, half_y: f32) -> PhysicsHandle {
        let rb = RigidBodyBuilder::new_static().build();
        let rb_handle = self.rb_set.insert(rb);
        let coll = ColliderBuilder::cuboid(half_x, half_y).density(4.0).build();
        let coll_handle = self.coll_set.insert_with_parent(coll, rb_handle,
            &mut self.rb_set);
        PhysicsHandle(rb_handle, coll_handle)
    }

    pub fn get_rb(&self, rb_handle: RigidBodyHandle) -> &RigidBody {
        self.rb_set.get(rb_handle).unwrap()
    }

    pub fn get_rb_mut(&mut self, rb_handle: RigidBodyHandle) -> &mut RigidBody {
        self.rb_set.get_mut(rb_handle).unwrap()
    }

    pub fn convert_rb_iso(&self, iso: Isometry2<Real>) -> (V2, f32) {
        (V2::new(iso.translation.vector.x, iso.translation.vector.y),
            iso.rotation.angle())
    }

    pub fn get_converted_rb_iso(&self, rb_handle: RigidBodyHandle) -> (V2, f32) {
        self.convert_rb_iso(*self.get_rb(rb_handle).position())
    }

    pub fn get_rb_draw_params(&self, handle: RigidBodyHandle, origin: V2) -> DrawParams {
        let (position, rotation) = self.get_converted_rb_iso(handle);
        DrawParams {
            position, rotation, scale: V2::one(), origin, color: Color::WHITE
        }
    }
}

impl State for Physics {
    fn update(&mut self, _ctx: &mut tetra::Context) -> tetra::Result {
        self.physics_pipeline.step(&Vector2::new(0.0, 0.0), &self.integration_params,
            &mut self.island_manager, &mut self.broad_phase, &mut self.narrow_phase,
            &mut self.rb_set, &mut self.coll_set, &mut self.joint_set,
            &mut self.ccd_solver, &(), &());
        Ok(())
    }
}
