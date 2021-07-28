use crossbeam_channel::{Receiver};
use rapier2d::{math::Real, na::{Isometry2}, prelude::{ActiveEvents, BroadPhase, CCDSolver, ChannelEventCollector, ColliderBuilder, ColliderHandle, ColliderSet, ContactEvent, IntegrationParameters, IntersectionEvent, IslandManager, JointSet, NarrowPhase, PhysicsPipeline, RigidBody, RigidBodyBuilder, RigidBodyHandle, RigidBodySet}};
use tetra::{State, graphics::{Color, DrawParams}, math::Vec2};
use crate::{EntityType, conv_vec};

pub const MASS_FORCE_SCALE: f32 = 1000.0;

pub type V2 = Vec2<f32>;

pub struct PhysicsHandle(pub RigidBodyHandle, pub ColliderHandle);

pub struct Physics {
    pub rb_set: RigidBodySet,
    pub coll_set: ColliderSet,
    pub wind: V2,
    integration_params: IntegrationParameters,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    joint_set: JointSet,
    ccd_solver: CCDSolver,
    intersection_receiver: Receiver<IntersectionEvent>,
    contact_receiver: Receiver<ContactEvent>,
    event_handler: ChannelEventCollector,
    physics_pipeline: PhysicsPipeline
}

impl Physics {
    pub fn setup() -> Physics {
        let (intersection_sender, intersection_receiver) = crossbeam_channel::unbounded();
        let (contact_sender, contact_receiver) = crossbeam_channel::unbounded();
        let event_handler = ChannelEventCollector::new(
            intersection_sender, contact_sender);
        Physics {
            rb_set: RigidBodySet::new(),
            coll_set: ColliderSet::new(),
            wind: V2::zero(),
            integration_params: IntegrationParameters::default(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            joint_set: JointSet::new(),
            ccd_solver: CCDSolver::new(),
            intersection_receiver,
            contact_receiver,
            event_handler,
            physics_pipeline: PhysicsPipeline::new()
        }
    }

    pub fn build_ship_collider(&mut self, half_x: f32, half_y: f32) -> PhysicsHandle {
        let rb = RigidBodyBuilder::new_dynamic()
            .linear_damping(1.5).angular_damping(2.5).build();
        let rb_handle = self.rb_set.insert(rb);
        let coll = ColliderBuilder::cuboid(half_x * 0.85, half_y * 0.85)
            .density(1.0).friction(2.0).restitution(0.8)
            .active_events(ActiveEvents::CONTACT_EVENTS | ActiveEvents::INTERSECTION_EVENTS)
            .user_data(EntityType::Ship.to_num()).build();
        let coll_handle = self.coll_set.insert_with_parent(coll, rb_handle,
            &mut self.rb_set);
        PhysicsHandle(rb_handle, coll_handle)
    }

    pub fn build_island_collider(&mut self, half_x: f32, half_y: f32) -> PhysicsHandle {
        let rb = RigidBodyBuilder::new_static().build();
        let rb_handle = self.rb_set.insert(rb);
        let coll = ColliderBuilder::cuboid(half_x, half_y).density(4.0)
            .active_events(ActiveEvents::CONTACT_EVENTS | ActiveEvents::INTERSECTION_EVENTS)
            .user_data(EntityType::Island.to_num()).build();
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

    pub fn get_intersections(&self) -> Vec<IntersectionEvent> {
        let mut events = Vec::new();
        while let Ok(ev) = self.intersection_receiver.try_recv() {
            events.push(ev);
        }
        events
    }

    pub fn get_contacts(&self) -> Vec<ContactEvent> {
        let mut events = Vec::new();
        while let Ok(ev) = self.contact_receiver.try_recv() {
            events.push(ev);
        }
        events
    }
}

impl State for Physics {
    fn update(&mut self, _ctx: &mut tetra::Context) -> tetra::Result {
        self.physics_pipeline.step(&conv_vec(self.wind), &self.integration_params,
            &mut self.island_manager, &mut self.broad_phase, &mut self.narrow_phase,
            &mut self.rb_set, &mut self.coll_set, &mut self.joint_set,
            &mut self.ccd_solver, &(), &self.event_handler);
        Ok(())
    }
}
