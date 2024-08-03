use std::marker::PhantomData;

use bevy::ecs::{
    component::{Component, ComponentId},
    entity::Entity,
};
use leptos::tachys::view::Render;
use next_tuple::NextTuple;

use super::{
    renderer::{with_world_and_nodes, LeptosBevy, LeptosNodeId},
    BevyElement,
};

// GENERAL BOILERPLATE

/// Trait for a property that can be applied to a node
pub trait Property {
    type State;

    fn build(self, element: &LeptosNodeId) -> Self::State;

    fn rebuild(self, element: &LeptosNodeId, state: &mut Self::State);
}

impl Property for () {
    type State = ();

    fn build(self, _element: &LeptosNodeId) -> Self::State {}

    fn rebuild(self, _element: &LeptosNodeId, _state: &mut Self::State) {}
}

macro_rules! tuples {
        ($($ty:ident),* $(,)?) => {

            impl<$($ty,)*> Property for ($($ty,)*)
                where $($ty: Property,)*
            {
                type State = ($($ty::State,)*);

                fn build(self, element: &LeptosNodeId) -> Self::State {
                    #[allow(non_snake_case)]
                    let ($($ty,)*) = self;
                    ($($ty.build(element),)*)
                }

                fn rebuild(self, element: &LeptosNodeId, state: &mut Self::State) {
                    paste::paste! {
                        #[allow(non_snake_case)]
                        let ($($ty,)*) = self;
                        #[allow(non_snake_case)]
                        let ($([<state_ $ty:lower>],)*) = state;
                        $($ty.rebuild(element, [<state_ $ty:lower>]));*
                    }
                }
            }
        }
    }

tuples!(A);
tuples!(A, B);
tuples!(A, B, C);
tuples!(A, B, C, D);
tuples!(A, B, C, D, E);
tuples!(A, B, C, D, E, F);
tuples!(A, B, C, D, E, F, G);
tuples!(A, B, C, D, E, F, G, H);
tuples!(A, B, C, D, E, F, G, H, I);
tuples!(A, B, C, D, E, F, G, H, I, J);
tuples!(A, B, C, D, E, F, G, H, I, J, K);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y
);

pub struct LComp<C: Component> {
    component: C,
}

pub struct LCompState;

impl<C: Component> Property for LComp<C> {
    type State = LCompState;

    fn build(self, element: &LeptosNodeId) -> Self::State {
        with_world_and_nodes(|world, node_map| {
            let node = node_map
                .get(element)
                .expect("Property::build() for LComp but no node.");
            let mut entity_mut = world.entity_mut(*node.entity());
            entity_mut.insert(self.component);

            LCompState
        })
    }

    fn rebuild(self, element: &LeptosNodeId, _state: &mut Self::State) {
        with_world_and_nodes(|world, node_map| {
            let node = node_map.get(element).expect(
                "Property::<C: Component>::rebuild() but no node.",
            );
            let mut entity_mut = world.entity_mut(*node.entity());
            entity_mut.insert(self.component);
            LCompState
        });
    }
}

pub struct LConditionalComp<C: Component> {
    component: Option<C>,
}

pub enum LConditionalCompState {
    Inserted,
    NotInserted,
}

impl<C: Component> Property for LConditionalComp<C> {
    type State = LConditionalCompState;

    fn build(self, element: &LeptosNodeId) -> Self::State {
        with_world_and_nodes(|world, node_map| match self.component {
            Some(component) => {
                let node = node_map.get(element).expect(
                    "Property::<C: Component>::build() but no node.",
                );
                let mut entity_mut = world.entity_mut(*node.entity());
                entity_mut.insert(component);

                Self::State::Inserted
            }
            None => Self::State::NotInserted,
        })
    }

    fn rebuild(self, element: &LeptosNodeId, state: &mut Self::State) {
        match (self.component, &state) {
            (Some(component), Self::State::Inserted) => {
                with_world_and_nodes(|world, node_map| {
                    let node = node_map.get(element).expect(
                        "Property::<C: Component>::build() but no node.",
                    );
                    let mut entity_mut = world.entity_mut(*node.entity());
                    entity_mut.insert(component);
                })
            }
            (None, Self::State::Inserted) => {
                with_world_and_nodes(|world, node_map| {
                    let node = node_map.get(element).expect(
                        "Property::<C: Component>::build() but no node.",
                    );
                    let mut entity_mut = world.entity_mut(*node.entity());
                    entity_mut.remove::<C>();
                    *state = Self::State::NotInserted;
                })
            }
            (Some(component), Self::State::NotInserted) => {
                with_world_and_nodes(|world, node_map| {
                    let node = node_map.get(element).expect(
                        "Property::<C: Component>::build() but no node.",
                    );
                    let mut entity_mut = world.entity_mut(*node.entity());
                    entity_mut.insert(component);
                    *state = Self::State::Inserted;
                })
            }
            (None, Self::State::NotInserted) => {}
        }
    }
}

impl<TProps, TChildren> BevyElement<Entity, TProps, TChildren>
where
    TProps: NextTuple,
    TChildren: Render<LeptosBevy>,
{
    pub fn component<C: Component>(
        self,
        value: C,
    ) -> BevyElement<Entity, TProps::Output<LComp<C>>, TChildren> {
        let BevyElement {
            ty,
            properties,
            children,
        } = self;
        BevyElement {
            ty,
            properties: properties.next_tuple(LComp { component: value }),
            children,
        }
    }

    pub fn opt_component<C: Component>(
        self,
        value: Option<C>,
    ) -> BevyElement<Entity, TProps::Output<LConditionalComp<C>>, TChildren> {
    let BevyElement {
        ty,
        properties,
        children,
    } = self;
    BevyElement {
        ty,
        properties: properties.next_tuple(LConditionalComp { component: value }),
        children,
    }
}
}
