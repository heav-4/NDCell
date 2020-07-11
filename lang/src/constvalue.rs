//! Values used by the interpreter for NDCA.

use std::rc::Rc;

use crate::errors::*;
use crate::types::{CellStateFilter, LangCellState, LangInt, Type};

/// Constant value of any type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstValue {
    /// Void.
    Void,
    /// Integer.
    Int(LangInt),
    /// Cell state.
    CellState(LangCellState),
    /// Vector of a specific length from 1 to 256.
    Vector(Vec<LangInt>),
    /// Inclusive integer range.
    IntRange {
        /// First number in the range.
        start: LangInt,
        /// Number to end at or before (inclusive).
        end: LangInt,
        /// Step (may be negative).
        step: LangInt,
    },
    /// Inclusive hyperrectangle, represented by the coordinates of two opposite corners.
    Rectangle(Vec<LangInt>, Vec<LangInt>),
    /// Cell state filter.
    CellStateFilter(CellStateFilter),

    /// String of text.
    String(Rc<String>),
}
impl ConstValue {
    /// Returns the type of this value.
    pub fn ty(&self) -> Type {
        match self {
            Self::Void => Type::Void,
            Self::Int(_) => Type::Int,
            Self::CellState(_) => Type::CellState,
            Self::Vector(values) => Type::Vector(values.len()),
            Self::IntRange { .. } => Type::IntRange,
            Self::Rectangle(start, _) => Type::Rectangle(start.len()),
            Self::CellStateFilter(f) => Type::CellStateFilter(f.state_count()),
            Self::String(_) => Type::String,
        }
    }
    /// Constructs a default value of the given type.
    pub fn default(ty: &Type) -> Self {
        match ty {
            Type::Void => Self::Void,
            Type::Int => Self::Int(0),
            Type::CellState => Self::CellState(0),
            Type::Vector(len) => Self::Vector(vec![0; *len]),
            Type::Pattern(_) => todo!("default pattern (all #0)"),
            // Default integer range includes only zero.
            Type::IntRange => Self::IntRange {
                start: 0,
                end: 0,
                step: 1,
            },
            // Default rectangle includes only the origin.
            Type::Rectangle(ndim) => Self::Rectangle(vec![0; *ndim], vec![0; *ndim]),
            // Default cell state filter includes no cells.
            Type::CellStateFilter(state_count) => {
                Self::CellStateFilter(CellStateFilter::none(*state_count))
            }
            // Default string is the empty string.
            Type::String => ConstValue::String(Rc::new(String::new())),
        }
    }

    /// Returns the integer value inside if this is a ConstValue::Int; otherwise
    /// returns an InternalError.
    pub fn as_int(self) -> LangResult<LangInt> {
        match self {
            Self::Int(i) => Ok(i),
            _ => uncaught_type_error!(),
        }
    }
    /// Returns the integer value inside if this is a ConstValue::CellState;
    /// otherwise returns an InternalError.
    pub fn as_cell_state(self) -> LangResult<LangCellState> {
        match self {
            Self::CellState(i) => Ok(i),
            _ => uncaught_type_error!(),
        }
    }
    /// Returns the vector value inside if this is a ConstValue::Vector;
    /// otherwise returns an InternalError.
    pub fn as_vector(self) -> LangResult<Vec<LangInt>> {
        match self {
            Self::Vector(v) => Ok(v),
            _ => uncaught_type_error!(),
        }
    }
    /// Returns the start, end, and step inside if this is a
    /// ConstValue::IntRange; otherwise returns an InternalError.
    pub fn as_int_range(self) -> LangResult<(LangInt, LangInt, LangInt)> {
        match self {
            Self::IntRange { start, end, step } => Ok((start, end, step)),
            _ => uncaught_type_error!(),
        }
    }
    /// Returns the start and end inside if this is a ConstValue::Rectangle;
    /// otherwise returns an InternalError.
    pub fn as_rectangle(self) -> LangResult<(Vec<LangInt>, Vec<LangInt>)> {
        match self {
            Self::Rectangle(start, end) => Ok((start, end)),
            _ => uncaught_type_error!(),
        }
    }
    /// Returns the value inside if this is a ConstValue::CellStateFilter;
    /// otherwise returns an InternalError.
    pub fn as_cell_state_filter(self) -> LangResult<CellStateFilter> {
        match self {
            Self::CellStateFilter(f) => Ok(f),
            _ => uncaught_type_error!(),
        }
    }
    /// Returns the value inside if this is a ConstValue::String; otherwise
    /// returns an InternalError.
    pub fn as_string(self) -> LangResult<Rc<String>> {
        match self {
            Self::String(s) => Ok(s),
            _ => uncaught_type_error!(),
        }
    }

    /// Converts this value to a boolean if it can be converted; otherwise
    /// returns an InternalError.
    pub fn to_bool(self) -> LangResult<bool> {
        match self {
            Self::Int(i) => Ok(i != 0),
            Self::CellState(i) => Ok(i != 0),
            Self::Vector(v) => Ok(v.into_iter().any(|i| i != 0)),
            Self::Void
            | Self::IntRange { .. }
            | Self::Rectangle { .. }
            | Self::CellStateFilter(_)
            | Self::String(_) => uncaught_type_error!(),
        }
    }
    /// Converts this value to a vector of the specified length if this is a
    /// ConstValue::Int or ConstValue::Vector; otherwise returns an
    /// InternalError.
    pub fn coerce_to_vector(self, len: usize) -> LangResult<Vec<LangInt>> {
        match self {
            Self::Int(i) => Ok(vec![i; len]),
            Self::Vector(mut v) => {
                if v.len() < len {
                    // Not long enough; extend with zeros.
                    v.extend(std::iter::repeat(0).take(len - v.len()));
                } else if v.len() > len {
                    // Too long; truncate.
                    v.truncate(len);
                }
                Ok(v)
            }
            _ => uncaught_type_error!(),
        }
    }
    /// Converts this value to a rectangle of the specified number of dimensions
    /// if this is a ConstValue::Int, ConstValue::Vector, ConstValue::IntRange,
    /// or ConstValue::Rectangle; otherwise returns an InternalError.
    pub fn coerce_to_rectangle(self, ndim: usize) -> LangResult<(Vec<LangInt>, Vec<LangInt>)> {
        match self {
            Self::Int(i) => Ok((vec![i; ndim], vec![i; ndim])),
            Self::Vector(v) => {
                let pos = Self::Vector(v).coerce_to_vector(ndim)?;
                Ok((pos.clone(), pos))
            }
            Self::IntRange { start, end, .. } => Ok((vec![start; ndim], vec![end; ndim])),
            Self::Rectangle(start, end) => Ok((
                Self::Vector(start).coerce_to_vector(ndim)?,
                Self::Vector(end).coerce_to_vector(ndim)?,
            )),
            _ => uncaught_type_error!(),
        }
    }
    /// Converts this value to a cell state filter if this is a
    /// ConstValue::CellState or ConstValue::CellStateFilter; otherwise returns
    /// an InternalError.
    pub fn coerce_to_cell_state_filter(self, state_count: usize) -> LangResult<CellStateFilter> {
        match self {
            Self::CellState(i) => Ok(CellStateFilter::single_cell_state(state_count, i)),
            Self::CellStateFilter(f) => Ok(f),
            _ => uncaught_type_error!(),
        }
    }

    /// Returns the range step to use by default given start and end integers.
    /// +1 if start <= end; -1 if start > end.
    pub fn infer_range_step(start: LangInt, end: LangInt) -> LangInt {
        if start <= end {
            1
        } else {
            -1
        }
    }
}
