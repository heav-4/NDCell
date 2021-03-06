//! N-dimensional cellular automaton simulation backend.
//!
//! This crate contains everything needed to store and simulate cellular
//! automata.

#![allow(dead_code)]
#![warn(missing_docs)]

#[macro_use]
extern crate pest_derive;

use enum_dispatch::enum_dispatch;
use num::BigInt;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

mod io;
pub mod math;
mod ndsimulate;
pub mod projection;
pub mod rule;
pub mod simulation;
pub mod space;

pub use io::*;
pub use ndsimulate::*;
pub use projection::*;
pub use rle::RleEncode;
pub use rule::{DummyRule, Rule, TransitionFunction};
pub use simulation::*;
pub use space::*;

/// ProjectedAutomaton functionality implemented by dispatching to
/// NdProjectedAutomaton.
#[enum_dispatch]
pub trait NdProjectedAutomatonTrait<P: Dim> {
    /// Returns the projected NdTree.
    fn get_projected_tree(&self) -> NdTree<u8, P>;
    /// Returns the ProjectionParams used to create this projection.
    fn get_projection_params(&self) -> ProjectionParams;
    /// Sets the projection from the ProjectionParams.
    fn set_projection_params(&mut self, params: ProjectionParams) -> Result<(), NdProjectionError>;
    /// Sets a cell using projected coordinates.
    fn set_cell(&mut self, pos: &BigVec<P>, state: u8);
}

/// A cellular automaton projected to 1D.
pub type ProjectedAutomaton1D = ProjectedAutomaton<Dim1D>;
/// A cellular automaton projected to 2D.
pub type ProjectedAutomaton2D = ProjectedAutomaton<Dim2D>;
/// A cellular automaton projected to 3D.
pub type ProjectedAutomaton3D = ProjectedAutomaton<Dim3D>;
/// A cellular automaton projected to 4D.
pub type ProjectedAutomaton4D = ProjectedAutomaton<Dim4D>;
/// A cellular automaton projected to 5D.
pub type ProjectedAutomaton5D = ProjectedAutomaton<Dim5D>;
/// A cellular automaton projected to 6D.
pub type ProjectedAutomaton6D = ProjectedAutomaton<Dim6D>;

/// An automaton of an unknown dimensionality combined with a projection to a
/// given dimensionality.
#[allow(missing_docs)]
#[enum_dispatch(NdProjectedAutomatonTrait)]
#[derive(Clone)]
pub enum ProjectedAutomaton<P: Dim> {
    From1D(NdProjectedAutomaton<Dim1D, P>),
    From2D(NdProjectedAutomaton<Dim2D, P>),
    From3D(NdProjectedAutomaton<Dim3D, P>),
    From4D(NdProjectedAutomaton<Dim4D, P>),
    From5D(NdProjectedAutomaton<Dim5D, P>),
    From6D(NdProjectedAutomaton<Dim6D, P>),
}
impl<P: Dim> Default for ProjectedAutomaton<P> {
    fn default() -> Self {
        let inner = Box::new(NdProjectedAutomaton::<P, P>::default());
        match P::NDIM {
            1 => Self::From1D(unsafe {
                *std::mem::transmute::<
                    Box<NdProjectedAutomaton<P, P>>,
                    Box<NdProjectedAutomaton<Dim1D, P>>,
                >(inner)
            }),
            2 => Self::From2D(unsafe {
                *std::mem::transmute::<
                    Box<NdProjectedAutomaton<P, P>>,
                    Box<NdProjectedAutomaton<Dim2D, P>>,
                >(inner)
            }),
            3 => Self::From3D(unsafe {
                *std::mem::transmute::<
                    Box<NdProjectedAutomaton<P, P>>,
                    Box<NdProjectedAutomaton<Dim3D, P>>,
                >(inner)
            }),
            4 => Self::From4D(unsafe {
                *std::mem::transmute::<
                    Box<NdProjectedAutomaton<P, P>>,
                    Box<NdProjectedAutomaton<Dim4D, P>>,
                >(inner)
            }),
            5 => Self::From5D(unsafe {
                *std::mem::transmute::<
                    Box<NdProjectedAutomaton<P, P>>,
                    Box<NdProjectedAutomaton<Dim5D, P>>,
                >(inner)
            }),
            6 => Self::From6D(unsafe {
                *std::mem::transmute::<
                    Box<NdProjectedAutomaton<P, P>>,
                    Box<NdProjectedAutomaton<Dim6D, P>>,
                >(inner)
            }),
            _ => unreachable!("Dimensions above 6 are not supported"),
        }
    }
}
impl<D: Dim, P: Dim> From<NdAutomaton<D>> for ProjectedAutomaton<P>
where
    NdProjectedAutomaton<D, P>: From<NdAutomaton<D>>,
    Self: From<NdProjectedAutomaton<D, P>>,
{
    fn from(automaton: NdAutomaton<D>) -> Self {
        Self::from(NdProjectedAutomaton::from(automaton))
    }
}
impl<P: Dim> IntoNdSimulate for ProjectedAutomaton<P> {
    fn ndsim(&self) -> &dyn NdSimulate {
        match self {
            Self::From1D(inner) => inner,
            Self::From2D(inner) => inner,
            Self::From3D(inner) => inner,
            Self::From4D(inner) => inner,
            Self::From5D(inner) => inner,
            Self::From6D(inner) => inner,
        }
    }
    fn ndsim_mut(&mut self) -> &mut dyn NdSimulate {
        match self {
            Self::From1D(inner) => inner,
            Self::From2D(inner) => inner,
            Self::From3D(inner) => inner,
            Self::From4D(inner) => inner,
            Self::From5D(inner) => inner,
            Self::From6D(inner) => inner,
        }
    }
}

/// A D-dimensional automaton with a projection from a D-dimensional grid to a
/// P-dimensional one.
#[allow(missing_docs)]
#[derive(Clone)]
pub struct NdProjectedAutomaton<D: Dim, P: Dim> {
    pub automaton: NdAutomaton<D>,
    pub projection: NdProjection<u8, D, P>,
}
impl<D: Dim> From<NdAutomaton<D>> for NdProjectedAutomaton<D, D> {
    fn from(automaton: NdAutomaton<D>) -> Self {
        Self {
            automaton,
            projection: Default::default(),
        }
    }
}
impl<D: Dim> Default for NdProjectedAutomaton<D, D> {
    fn default() -> Self {
        Self::from(NdAutomaton::default())
    }
}
impl<D: Dim, P: Dim> IntoNdSimulate for NdProjectedAutomaton<D, P> {
    fn ndsim(&self) -> &dyn NdSimulate {
        &self.automaton
    }
    fn ndsim_mut(&mut self) -> &mut dyn NdSimulate {
        &mut self.automaton
    }
}
impl<D: Dim, P: Dim> NdProjectedAutomatonTrait<P> for NdProjectedAutomaton<D, P> {
    fn get_projected_tree(&self) -> NdTree<u8, P> {
        self.projection.project(&self.automaton.tree)
    }
    fn get_projection_params(&self) -> ProjectionParams {
        self.projection.get_params()
    }
    fn set_projection_params(&mut self, params: ProjectionParams) -> Result<(), NdProjectionError> {
        self.projection = NdProjection(params.try_into()?);
        Ok(())
    }
    fn set_cell(&mut self, pos: &BigVec<P>, state: u8) {
        self.automaton
            .tree
            .set_cell(&self.projection.unproject_pos(pos), state);
    }
}

/// A fully-fledged cellular automaton, including a grid (NdTree), rule
/// (Simulation), and generation count.
#[allow(missing_docs)]
#[derive(Clone, Default)]
pub struct NdAutomaton<D: Dim> {
    pub tree: NdTree<u8, D>,
    pub sim: Arc<Mutex<Simulation<u8, D>>>,
    pub generations: BigInt,
}
impl<D: Dim> NdSimulate for NdAutomaton<D> {
    fn get_ndim(&self) -> usize {
        D::NDIM
    }
    fn get_population(&self) -> &BigInt {
        &self.tree.get_root().population
    }
    fn get_generation_count(&self) -> &BigInt {
        &self.generations
    }
    fn set_generation_count(&mut self, generations: BigInt) {
        self.generations = generations;
    }
    fn step(&mut self, step_size: &BigInt) {
        self.sim.lock().unwrap().step(&mut self.tree, step_size);
        self.generations += step_size;
    }
}
impl<D: Dim> NdAutomaton<D> {
    /// Sets the simulation of this automaton.
    pub fn set_sim(&mut self, new_sim: Simulation<u8, D>) {
        self.sim = Arc::new(Mutex::new(new_sim));
    }
}

/// A 1D cellular automaton.
pub type Automaton1D = NdAutomaton<Dim1D>;
/// A 2D cellular automaton.
pub type Automaton2D = NdAutomaton<Dim2D>;
/// A 3D cellular automaton.
pub type Automaton3D = NdAutomaton<Dim3D>;
/// A 4D cellular automaton.
pub type Automaton4D = NdAutomaton<Dim4D>;
/// A 5D cellular automaton.
pub type Automaton5D = NdAutomaton<Dim5D>;
/// A 6D cellular automaton.
pub type Automaton6D = NdAutomaton<Dim6D>;

/// An immutable reference to a cellular automaton of an unknown dimensionality.
#[allow(missing_docs)]
pub enum Automaton<'a> {
    Automaton1D(&'a Automaton1D),
    Automaton2D(&'a Automaton2D),
    Automaton3D(&'a Automaton3D),
    Automaton4D(&'a Automaton4D),
    Automaton5D(&'a Automaton5D),
    Automaton6D(&'a Automaton6D),
}
impl<'a, P: Dim> From<&'a ProjectedAutomaton<P>> for Automaton<'a> {
    fn from(projected_automaton: &'a ProjectedAutomaton<P>) -> Self {
        match projected_automaton {
            ProjectedAutomaton::From1D(inner) => Self::Automaton1D(&inner.automaton),
            ProjectedAutomaton::From2D(inner) => Self::Automaton2D(&inner.automaton),
            ProjectedAutomaton::From3D(inner) => Self::Automaton3D(&inner.automaton),
            ProjectedAutomaton::From4D(inner) => Self::Automaton4D(&inner.automaton),
            ProjectedAutomaton::From5D(inner) => Self::Automaton5D(&inner.automaton),
            ProjectedAutomaton::From6D(inner) => Self::Automaton6D(&inner.automaton),
        }
    }
}

/// A mutable reference to a cellular automaton of an unknown dimensionality.
#[allow(missing_docs)]
pub enum AutomatonMut<'a> {
    Automaton1D(&'a mut Automaton1D),
    Automaton2D(&'a mut Automaton2D),
    Automaton3D(&'a mut Automaton3D),
    Automaton4D(&'a mut Automaton4D),
    Automaton5D(&'a mut Automaton5D),
    Automaton6D(&'a mut Automaton6D),
}
impl<'a, P: Dim> From<&'a mut ProjectedAutomaton<P>> for AutomatonMut<'a> {
    fn from(projected_automaton: &'a mut ProjectedAutomaton<P>) -> Self {
        match projected_automaton {
            ProjectedAutomaton::From1D(inner) => Self::Automaton1D(&mut inner.automaton),
            ProjectedAutomaton::From2D(inner) => Self::Automaton2D(&mut inner.automaton),
            ProjectedAutomaton::From3D(inner) => Self::Automaton3D(&mut inner.automaton),
            ProjectedAutomaton::From4D(inner) => Self::Automaton4D(&mut inner.automaton),
            ProjectedAutomaton::From5D(inner) => Self::Automaton5D(&mut inner.automaton),
            ProjectedAutomaton::From6D(inner) => Self::Automaton6D(&mut inner.automaton),
        }
    }
}

#[cfg(test)]
mod tests;
