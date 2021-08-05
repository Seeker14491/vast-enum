//! A wrapper for fieldless enums that allows representing invalid enum discriminants.
//!
//! The wrapped enum must implement `Into<Repr>`, where `Repr` is a primitive integer, and `Repr`
//! must implement `TryInto<Enum>`. These impls can be easily derived using the [num_enum][1] crate,
//! as shown in the example below.
//!
//! # Example
//!
//! ```
//! use num_enum::{IntoPrimitive, TryFromPrimitive};
//! use vast_enum::VastEnum;
//!
//! #[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
//! #[repr(u8)]
//! enum Color {
//!     Red = 0,
//!     Yellow = 1,
//!     Green = 2,
//! }
//!
//! let mut enum_ = VastEnum::from_variant(Color::Green);
//! assert!(enum_.is_valid());
//! assert_eq!(enum_.int(), 2);
//!
//! *enum_.int_mut() = 1;
//! assert!(enum_.is_valid());
//! assert_eq!(enum_.variant(), Some(Color::Yellow));
//!
//! *enum_.int_mut() = 5;
//! assert!(!enum_.is_valid());
//! assert_eq!(enum_.variant(), None);
//! ```
//!
//! [1]: https://crates.io/crates/num_enum

#![no_std]

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use core::borrow::Borrow;
use core::convert::TryInto;
use core::fmt::{Debug, Formatter};
use core::hash::Hash;
use core::marker::PhantomData;
use derivative::Derivative;

/// A wrapper for fieldless enums that allows representing invalid enum discriminants.
///
/// This struct has the same in-memory representation as `Repr`, which represents the enum's integer
/// discriminant.
#[repr(transparent)]
#[derive(Derivative)]
#[derivative(
    Copy(bound = ""),
    Clone(bound = ""),
    Default(bound = ""),
    Hash(bound = ""),
    Eq(bound = ""),
    PartialEq(bound = ""),
    Ord(bound = ""),
    PartialOrd(bound = "")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VastEnum<Enum, Repr>(
    Repr,
    #[cfg_attr(feature = "serde", serde(skip))] PhantomData<Enum>,
)
where
    Enum: Into<Repr>,
    Repr: EnumRepr<Enum>;

impl<Enum, Repr> VastEnum<Enum, Repr>
where
    Enum: Into<Repr>,
    Repr: EnumRepr<Enum>,
{
    /// Creates a [`VastEnum`] from an integer discriminant.
    pub fn from_int(discriminant: Repr) -> Self {
        VastEnum(discriminant, PhantomData)
    }

    /// Returns the enum's integer discriminant.
    pub fn int(self) -> Repr {
        self.0
    }

    /// Returns a mutable reference to the enum's integer discriminant.
    pub fn int_mut(&mut self) -> &mut Repr {
        &mut self.0
    }

    /// Allows casting to a [`VastEnum`] with a different enum type.
    ///
    /// Equivalent to
    ///
    /// ```rust
    /// VastEnum::from_int(vast_enum.int())
    /// ```
    pub fn cast<EnumNew>(self) -> VastEnum<EnumNew, Repr>
    where
        EnumNew: Into<Repr>,
        Repr: EnumRepr<EnumNew>,
    {
        VastEnum::from_int(self.0)
    }

    /// Creates a [`VastEnum`] from an enum variant.
    pub fn from_variant(variant: Enum) -> Self {
        VastEnum(variant.into(), PhantomData)
    }

    /// Returns the wrapped enum type corresponding to the current integer discriminant, if the
    /// integer is a valid value for that enum type.
    pub fn variant(self) -> Option<Enum> {
        self.0.try_into().ok()
    }

    /// Returns whether the current integer discriminant is a valid value for the wrapped enum type.
    pub fn is_valid(self) -> bool {
        self.variant().is_some()
    }

    /// Transforms the wrapped enum using the provided closure.
    pub fn map<EnumOut>(self, f: impl FnOnce(Enum) -> EnumOut) -> VastEnum<EnumOut, Repr>
    where
        EnumOut: Into<Repr>,
        Repr: EnumRepr<EnumOut>,
    {
        match self.variant() {
            Some(enum_) => VastEnum::from_variant(f(enum_)),
            None => self.cast(),
        }
    }
}

impl<Enum, Repr> From<Enum> for VastEnum<Enum, Repr>
where
    Enum: Into<Repr>,
    Repr: EnumRepr<Enum>,
{
    fn from(enum_: Enum) -> Self {
        VastEnum::from_variant(enum_)
    }
}

impl<Enum, Repr> AsRef<Repr> for VastEnum<Enum, Repr>
where
    Enum: Into<Repr>,
    Repr: EnumRepr<Enum>,
{
    fn as_ref(&self) -> &Repr {
        &self.0
    }
}

impl<Enum, Repr> AsMut<Repr> for VastEnum<Enum, Repr>
where
    Enum: Into<Repr>,
    Repr: EnumRepr<Enum>,
{
    fn as_mut(&mut self) -> &mut Repr {
        &mut self.0
    }
}

impl<Enum, Repr> Borrow<Repr> for VastEnum<Enum, Repr>
where
    Enum: Into<Repr>,
    Repr: EnumRepr<Enum>,
{
    fn borrow(&self) -> &Repr {
        &self.0
    }
}

impl<Enum, Repr> Debug for VastEnum<Enum, Repr>
where
    Enum: Debug + Into<Repr>,
    Repr: Debug + EnumRepr<Enum>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut tuple = f.debug_tuple("VastEnum");
        match self.variant() {
            Some(enum_) => {
                tuple.field(&format_args!("{:?}: {:?}", self.0, enum_));
            }
            None => {
                tuple.field(&self.0);
            }
        }

        tuple.finish()
    }
}

/// A wrapper for traits that valid enum reprs implement.
pub trait EnumRepr<Enum>: Copy + Default + Hash + Eq + Ord + TryInto<Enum> {}

impl<Enum, Repr> EnumRepr<Enum> for Repr where Repr: Copy + Default + Hash + Eq + Ord + TryInto<Enum>
{}
