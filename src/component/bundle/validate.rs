//! Validator for [`ComponentBundle`](crate::component::bundle::ComponentBundle)s.


#[cfg(any(debug_assertions, feature = "keep_debug_names"))]
use crate::util::unqualified::UnqualifiedTypeName;
use core::any::TypeId;
#[cfg(any(debug_assertions, feature = "keep_debug_names"))]
use core::any::type_name;
use core::fmt;
use core::cmp::Ordering;
use alloc::collections::BTreeSet;


/// A container that stores the types included in a [`ComponentBundle`](crate::component::bundle::ComponentBundle).
pub struct BundleValidator {

    /// The [`Component`](crate::component::Component) types that the [`ComponentBundle`](crate::component::bundle::ComponentBundle) contains, how they are contained, and whether they conflict.
    entries : BTreeSet<BundleValidatorEntry>

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
impl PartialOrd for BundleValidatorEntry {
    fn partial_cmp(&self, other : &Self) -> Option<Ordering> {
        Some(Ord::cmp(&self, &other))
    }
}
impl Ord for BundleValidatorEntry {
    fn cmp(&self, other : &Self) -> Ordering {
        Ord::cmp(&self.id, &other.id)
    }
}

/// The state of a single entry in a [`BundleValidator`].
#[derive(Clone, Copy)]
enum BundleValidatorEntryState {

    /// The [`ComponentBundle`](crate::component::bundle::ComponentBundle) contains the [`Component`](crate::component::Component) type.
    Included,

    /// The [`ComponentBundle`](crate::component::bundle::ComponentBundle) contains the [`Component`](crate::component::Component) type more than once.
    IncludedError

}


impl BundleValidator {

    /// Creates an empty [`BundleValidator`].
    ///
    /// No values are requested by the [`ComponentBundle`](crate::component::bundle::ComponentBundle).
    pub fn empty() -> Self { Self {
        entries : BTreeSet::new()
    } }

    /// Creates a new [`BundleValidator`] from a type `T` and [`BundleValidatorEntryState`].
    fn of<T : 'static>(
        state : BundleValidatorEntryState
    ) -> Self {
        let mut entries = BTreeSet::new();
        entries.insert(BundleValidatorEntry::of::<T>(
            state
        ));
        Self { entries }
    }

    /// Creates a [`BundleValidator`] with a single entry.
    ///
    /// The [`ComponentBundle`](crate::component::bundle::ComponentBundle) requests a value of type `T`.
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
        if (self.entries.iter().any(|entry| entry.state.is_error())) {
            panic!("{}", self);
        }
    }

}

impl fmt::Display for BundleValidator {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut has_errors = false;
        for entry in &self.entries {
            if (entry.state.is_error()) {
                if (! has_errors) {
                    write!(f, "Bundle violates the archetype rules:")?;
                }
                has_errors = true;
            }
            match (entry.state) {
                BundleValidatorEntryState::IncludedError => {
                    #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
                    // SAFETY: `entry.name` is a value previously generated using `core::any::type_name`.
                    { write!(f, "\n  Already included {}", unsafe{ UnqualifiedTypeName::from_unchecked(entry.name) })?; }
                    #[cfg(not(any(debug_assertions, feature = "keep_debug_names")))]
                    { write!(f, "\n  Already included item")?; }
                },
                _ => { }
            }
        }
        if (! has_errors) {
            write!(f, "Query OK")?;
        }
        Ok(())
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

    fn is_error(&self) -> bool {
        match (self) {
            BundleValidatorEntryState::Included      => false,
            BundleValidatorEntryState::IncludedError => true
        }
    }

}
