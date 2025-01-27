//! Validator for [`ComponentBundle`](crate::component::ComponentBundle)s.


#[cfg(any(debug_assertions, feature = "keep_debug_names"))]
use crate::util::unqualified::UnqualifiedTypeName;
use core::any::TypeId;
#[cfg(any(debug_assertions, feature = "keep_debug_names"))]
use core::any::type_name;
use core::hash::{ Hash, Hasher };
use std::collections::HashSet;


/// A container that stores the types included in a [`ComponentBundle`](crate::component::ComponentBundle).
pub struct BundleValidator {

    /// The [`Component`](crate::component::Component) types that the [`ComponentBundle`](crate::component::ComponentBundle) contains, how they are contained, and whether they conflict.
    entries : HashSet<BundleValidatorEntry>

}

/// A single entry in a [`BundleValidator`].
#[derive(Clone, Copy)]
struct BundleValidatorEntry {

    /// The [`TypeId`] of the [`Component`](crate::component::Component) type.
    id    : TypeId,

    /// The [`type_name`] of the [`Component`](crate::component::Component) type.
    #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
    name  : &'static str,

    /// How the [`Component`](crate::component::Component) is contained, and whether it conflicts.
    state : BundleValidatorEntryState

}
impl PartialEq for BundleValidatorEntry {
    fn eq(&self, other : &Self) -> bool {
        PartialEq::eq(&self.id, &other.id)
    }
}
impl Eq for BundleValidatorEntry { }
impl Hash for BundleValidatorEntry {
    fn hash<H : Hasher>(&self, state : &mut H) {
        Hash::hash::<H>(&self.id, state)
    }
}

/// The state of a single entry in a [`BundleValidator`].
#[derive(Clone, Copy)]
enum BundleValidatorEntryState {

    /// The [`ComponentBundle`](crate::component::ComponentBundle) contains the [`Component`](crate::component::Component) type.
    Included,

    /// The [`ComponentBundle`](crate::component::ComponentBundle) contains the [`Component`](crate::component::Component) type more than once.
    IncludedError

}


impl BundleValidator {

    /// Creates an empty [`BundleValidator`].
    ///
    /// No values are requested by the [`ComponentBundle`](crate::component::ComponentBundle).
    pub fn empty() -> Self { Self {
        entries : HashSet::new()
    } }

    /// Creates a new [`BundleValidator`] from a type `T` and [`BundleValidatorEntryState`].
    fn of<T : 'static>(
        state : BundleValidatorEntryState
    ) -> Self {
        let mut entries = HashSet::with_capacity(1);
        entries.insert(BundleValidatorEntry::of::<T>(
            state
        ));
        Self { entries }
    }

    /// Creates a [`BundleValidator`] with a single entry.
    ///
    /// The [`ComponentBundle`](crate::component::ComponentBundle) requests a value of type `T`.
    pub fn of_included<T : 'static>() -> Self {
        Self::of::<T>(BundleValidatorEntryState::Included
    ) }

    /// Joins two [`BundleValidator`]s together.
    ///
    /// If any entries conflict, an error is stored and will be included in the panic message from [`BundleValidator::panic_on_violation`].
    pub fn join(mut a : Self, b : Self) -> Self {
        for mut b_entry in b.entries {
            if let Some(a_entry) = a.entries.get(&b_entry) {
                b_entry.state = BundleValidatorEntryState::join(a_entry.state, b_entry.state);
                a.entries.replace(b_entry);
            } else {
                #[cfg(debug_assertions)]
                if (! a.entries.insert(b_entry)) { unreachable!(); }
                #[cfg(not(debug_assertions))]
                a.entries.insert(b_entry);
            }
        }
        a
    }

    /// Ensures that no requested values conflict with each other.
    ///
    /// # Panics
    /// Panics if any requested values conflict with each other.
    #[track_caller]
    pub fn panic_on_violation(&self) {
        let mut has_errors = false;
        let mut errors     = String::new();
        for entry in &self.entries {
            match (entry.state) {
                BundleValidatorEntryState::IncludedError => {
                    has_errors = true;
                    #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
                    // SAFETY: `entry.name` is a value previously generated using `core::any::type_name`.
                    { errors += &format!("\n  Already included {}", unsafe{ UnqualifiedTypeName::from_unchecked(entry.name) }); }
                    #[cfg(not(any(debug_assertions, feature = "keep_debug_names")))]
                    { errors += &format!("\n  Already included item"); }
                },
                _ => { }
            }
        }
        if (has_errors) {
            panic!("Bundle violates the archetype rules:{}", errors);
        }
    }

}

impl BundleValidatorEntry {

    /// Creates a new [`BundleValidatorEntry`] from a type `T` and [`BundleValidatorEntryState`].
    fn of<T : 'static>(
        state : BundleValidatorEntryState
    ) -> Self { BundleValidatorEntry {
        id    : TypeId::of::<T>(),
        #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
        name  : type_name::<T>(),
        state
    } }

}

impl BundleValidatorEntryState {

    /// Joins two [`BundleValidatorEntryState`]s together, or switches to an error if they conflict.
    fn join(a : Self, b : Self) -> Self {
        match ((a, b)) {
            ( Self::IncludedError , _ ) | ( _ , Self::IncludedError ) => Self::IncludedError,
            ( Self::Included , Self::Included ) => Self::IncludedError
        }
    }

}
