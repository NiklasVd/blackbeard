use crossbeam_channel::{Receiver};
use rapier2d::{math::Real, na::{Isometry2}, prelude::{ActiveEvents, BroadPhase, CCDSolver, ChannelEventCollector, Collider, ColliderBuilder, ColliderHandle, ColliderSet, ContactEvent, Cuboid, IntegrationParameters, InteractionGroups, IntersectionEvent, IslandManager, JointSet, NarrowPhase, PhysicsPipeline, QueryPipeline, Ray, RigidBody, RigidBodyBuilder, RigidBodyHandle, RigidBodySet}};
use tetra::{State, graphics::{Color, DrawParams}, math::{Vec2}};
use crate::{conv_vec, conv_vec_point, entity::EntityType};

pub const MASS_FORCE_SCALE: f32 = 1000.0;

pub type V2 = Vec2<f32>;

#[derive(Clone, Copy)]
pub struct PhysicsHandle(pub RigidBodyHandle, pub ColliderHandle);

pub struct Physics {
    pub rb_set: RigidBodySet,
    pub coll_set: ColliderSet,
    pub wind: V2,
    pub integration_params: IntegrationParameters,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    joint_set: JointSet,
    ccd_solver: CCDSolver,
    intersection_receiver: Receiver<IntersectionEvent>,
    contact_receiver: Receiver<ContactEvent>,
    event_handler: ChannelEventCollector,
    physics_pipeline: PhysicsPipeline,
    query_pipeline: QueryPipeline
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
            physics_pipeline: PhysicsPipeline::new(),
            query_pipeline: QueryPipeline::new()
        }
    }

    pub fn build_ship_collider(&mut self, half_x: f32, half_y: f32, mass: f32)
        -> PhysicsHandle {
        let rb = RigidBodyBuilder::new_dynamic()
            .linear_damping(2.5).angular_damping(3.0).build();
        let rb_handle = self.rb_set.insert(rb);
        let coll = ColliderBuilder::cuboid(half_x * 0.9, half_y * 0.835)
            .density(mass).friction(2.0).restitution(0.8)
            .active_events(ActiveEvents::CONTACT_EVENTS | ActiveEvents::INTERSECTION_EVENTS)
            .collision_groups(InteractionGroups::new(
                get_any_coll_group(), get_any_coll_group()))
            .user_data(EntityType::Ship.to_num()).build();
        let coll_handle = self.coll_set.insert_with_parent(coll, rb_handle,
            &mut self.rb_set);
        
        PhysicsHandle(rb_handle, coll_handle)
    }

    pub fn build_static_collider(&mut self, density: f32, half_x: f32, half_y: f32,
        entity_type: EntityType) -> PhysicsHandle {
        let rb = RigidBodyBuilder::new_static().build();
        let rb_handle = self.rb_set.insert(rb);
        let coll = ColliderBuilder::cuboid(half_x, half_y).density(density)
            .active_events(ActiveEvents::CONTACT_EVENTS | ActiveEvents::INTERSECTION_EVENTS)
            .collision_groups(InteractionGroups::new(
                get_any_coll_group(), get_any_coll_group()))
            .user_data(entity_type.to_num()).build();
        let coll_handle = self.coll_set.insert_with_parent(coll, rb_handle,
            &mut self.rb_set);
        PhysicsHandle(rb_handle, coll_handle)
    }

    pub fn build_object_collider(&mut self, half_x: f32, half_y: f32) -> PhysicsHandle {
       self.build_static_collider(4.0, half_x, half_y, EntityType::Object)
    }

    pub fn build_harbour_collider(&mut self, half_x: f32, half_y: f32) -> PhysicsHandle {
        self.build_static_collider(4.0, half_x, half_y, EntityType::Harbour)
    }

    pub fn build_harbour_zone(&mut self, pos: V2, rot: f32, half_x: f32, half_y: f32)
        -> ColliderHandle {
        let mut rb = RigidBodyBuilder::new_static().build();
        rb.set_position(Isometry2::new(conv_vec(pos), rot), true);
        let rb_handle = self.rb_set.insert(rb);
        let coll = ColliderBuilder::cuboid(half_x, half_y)
            .sensor(true).user_data(EntityType::Harbour.to_num())
            .active_events(ActiveEvents::INTERSECTION_EVENTS)
            .build();
        let coll_handle = self.coll_set.insert_with_parent(coll, rb_handle,
            &mut self.rb_set);
        coll_handle
    }

    pub fn build_cannon_ball(&mut self, size: f32) -> PhysicsHandle {
        let rb = RigidBodyBuilder::new_dynamic().ccd_enabled(true)
            .linear_damping(5.0).angular_damping(5.0).build();
        let rb_handle = self.rb_set.insert(rb);
        let coll = ColliderBuilder::ball(size).active_events(ActiveEvents::CONTACT_EVENTS)
            .user_data(EntityType::CannonBall.to_num())
            .collision_groups(InteractionGroups::new(
                get_any_coll_group(), get_any_coll_group()))
            .density(0.1).build();
        let coll_handle = self.coll_set.insert_with_parent(coll, rb_handle, &mut self.rb_set);
        PhysicsHandle(rb_handle, coll_handle)
    }

    pub fn remove_collider(&mut self, handle: PhysicsHandle) {
        self.remove_collider_by_rb(handle.0)
    }

    pub fn remove_collider_by_rb(&mut self, handle: RigidBodyHandle) {
        self.rb_set.remove(handle, &mut self.island_manager, &mut self.coll_set,
            &mut self.joint_set);
    }

    pub fn clear_colliders(&mut self) {
        let rbs: Vec<_> = self.rb_set.iter().map(|(handle, _)| handle.clone()).collect();
        for rb in rbs {
            self.remove_collider_by_rb(rb);
        }
    }

    pub fn get_coll(&self, coll_handle: ColliderHandle) -> &Collider {
        self.coll_set.get(coll_handle).unwrap()
    }

    pub fn get_coll_mut(&mut self, coll_handle: ColliderHandle) -> &mut Collider {
        self.coll_set.get_mut(coll_handle).unwrap()
    }

    pub fn get_coll_type(&self, coll_handle: ColliderHandle) -> EntityType {
        EntityType::to_entity_type(self.get_coll(coll_handle).user_data)
    }

    pub fn set_coll_group(&mut self, handle: ColliderHandle, member: u32, filter: u32) {
        self.get_coll_mut(handle).set_collision_groups(InteractionGroups::new(
            member, filter))
    }

    pub fn get_rb(&self, rb_handle: RigidBodyHandle) -> &RigidBody {
        self.rb_set.get(rb_handle).unwrap()
    }

    pub fn get_rb_mut(&mut self, rb_handle: RigidBodyHandle) -> &mut RigidBody {
        self.rb_set.get_mut(rb_handle).unwrap()
    }

    pub fn set_translation(&mut self, rb_handle: RigidBodyHandle, pos: V2, rot: f32) {
        let rb = self.get_rb_mut(rb_handle);
        rb.set_position(Isometry2::new(conv_vec(pos), rot), true)
    }

    pub fn convert_iso_to_translation(&self, iso: Isometry2<Real>) -> (V2, f32) {
        (V2::new(iso.translation.vector.x, iso.translation.vector.y),
            iso.rotation.angle())
    }

    pub fn get_translation(&self, rb_handle: RigidBodyHandle) -> (V2, f32) {
        self.convert_iso_to_translation(*self.get_rb(rb_handle).position())
    }

    pub fn get_rb_draw_params(&self, handle: RigidBodyHandle, origin: V2) -> DrawParams {
        let (position, rotation) = self.get_translation(handle);
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

    pub fn cast_ray(&self, ray: Ray, dist: f32) -> Option<ColliderHandle> {
        if let Some((coll_handle, _)) = self.query_pipeline.cast_ray(
            &self.coll_set, &ray, dist, false,InteractionGroups::all(), None) {
            Some(coll_handle)
        } else {
            None
        }
    }

    pub fn cast_ray2(&self, from: V2, dir: V2, dist: f32) -> Option<ColliderHandle> {
        let ray = Ray::new(conv_vec_point(from), conv_vec(dir));
        self.cast_ray(ray, dist)
    }

    pub fn cast_cuboid(&self, from: V2, size: V2, dir: V2, dist: f32) -> Option<ColliderHandle> {
        let cuboid = Cuboid::new(conv_vec(size));
        if let Some((coll_handle, hit)) = self.query_pipeline.cast_shape(
            &self.coll_set, &Isometry2::new(conv_vec(from), 0.0), &conv_vec(dir), &cuboid,
            dist, InteractionGroups::all(), None) {
            Some(coll_handle)
        } else {
            None
        }
    }

    pub fn check_for_space(&self, mut at: V2, size: V2, dir: V2) -> V2 {
        let rel_dir = size * dir.normalized();
        let dist = rel_dir.magnitude();
        while let Some(obstacle) = self.cast_cuboid(at, size, dir, dist) {
            // Hit an object, try again
            at += rel_dir;
        }
        at
    }
}

impl State for Physics {
    fn update(&mut self, ctx: &mut tetra::Context) -> tetra::Result {
        self.physics_pipeline.step(&conv_vec(self.wind), &self.integration_params,
            &mut self.island_manager, &mut self.broad_phase, &mut self.narrow_phase,
            &mut self.rb_set, &mut self.coll_set, &mut self.joint_set,
            &mut self.ccd_solver, &(), &self.event_handler);
        self.query_pipeline.update(&mut self.island_manager, &self.rb_set, &self.coll_set);
        Ok(())
    }
}

pub fn get_any_coll_group() -> u32 {
    1 | 2 | 4
}

pub fn get_decal_coll_group() -> u32 {
    6
}

pub fn get_empty_coll_group() -> u32 {
    0
}
