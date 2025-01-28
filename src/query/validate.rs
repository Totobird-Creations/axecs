#[cfg(any(debug_assertions, feature = "keep_debug_names"))]
use crate::util::unqualified::UnqualifiedTypeName;
use core::any::TypeId;
#[cfg(any(debug_assertions, feature = "keep_debug_names"))]
use core::any::type_name;
use core::fmt;
use core::cmp::Ordering;
use alloc::collections::BTreeSet;


/// A container that stores the types requested by a [`Query`](crate::query::Query).
pub struct QueryValidator {

    /// The value types that the [`Query`](crate::query::Query) accesses, how they are accessed, and whether they conflict.
    entries : BTreeSet<QueryValidatorEntry>

}

#[derive(Clone, Copy)]
struct QueryValidatorEntry {

    /// The [`TypeId`] of the value.
    id    : TypeId,

    /// The [`type_name`] of the value.
    #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
    #[doc(cfg(feature = "keep_debug_names"))]
    name  : &'static str,

    /// How the value is accessed, and whether it conflicts.
    state : QueryValidatorEntryState

}
impl PartialEq for QueryValidatorEntry {
    fn eq(&self, other : &Self) -> bool {
        PartialEq::eq(&self.id, &other.id)
    }
}
impl Eq for QueryValidatorEntry { }
impl PartialOrd for QueryValidatorEntry {
    fn partial_cmp(&self, other : &Self) -> Option<Ordering> {
        Some(Ord::cmp(&self, &other))
    }
}
impl Ord for QueryValidatorEntry {
    fn cmp(&self, other : &Self) -> Ordering {
        Ord::cmp(&self.id, &other.id)
    }
}

#[derive(Clone, Copy)]
enum QueryValidatorEntryState {

    /// The value is accessed some number of times immutably.
    Immutable,

    /// The value is accessed once, mutably.
    Mutable,

    /// The value is accessed once, ownership-taken.
    Owned,

    /// The value is accesed mutably and another way.
    MutableError,

    /// The value is ownership-taken and accessed another way
    OwnedError

}


impl QueryValidator {

    /// Creates an empty [`QueryValidator`].
    ///
    /// No values are requested by the [`Query`](crate::query::Query).
    pub fn empty() -> Self { Self {
        entries : BTreeSet::new()
    } }

    /// Creates a new [`QueryValidator`] from a type `T` and [`QueryValidatorEntryState`].
    fn of<T : 'static>(
        state : QueryValidatorEntryState
    ) -> Self {
        let mut entries = BTreeSet::new();
        entries.insert(QueryValidatorEntry::of::<T>(state));
        Self { entries }
    }

    /// Creates a [`QueryValidator`] with a single entry.
    ///
    /// The [`Query`](crate::query::Query) requests immutable access to a value of type `T`.
    ///
    /// Any number of immutable references to a type are allowed at the same time.
    pub fn of_immutable<T : 'static>() -> Self {
        Self::of::<T>(QueryValidatorEntryState::Immutable
    ) }

    /// Creates a [`QueryValidator`] with a single entry.
    ///
    /// The [`Query`](crate::query::Query) requests mutable access to a value of type `T`.
    ///
    /// A single mutable reference to a type is allowed. No other access of any type is allowed at the same time.
    pub fn of_mutable<T : 'static>() -> Self {
        Self::of::<T>(QueryValidatorEntryState::Mutable)
    }

    /// Creates a [`QueryValidator`] with a single entry.
    ///
    /// The [`Query`](crate::query::Query) requests ownership of a value of type `T`.
    ///
    /// A single ownership access to a type is allowed. No other access of any type is allowed at the same time.
    pub fn of_owned<T : 'static>() -> Self { Self::of::<T>(
        QueryValidatorEntryState::Owned
    ) }

    /// Joins two [`QueryValidator`]s together.
    ///
    /// If any entries conflict, an error is stored and will be included in the panic message from [`QueryValidator::panic_on_violation`].
    pub fn join(mut a : Self, b : Self) -> Self {
        for mut b_entry in b.entries {
            if let Some(a_entry) = a.entries.get(&b_entry) {
                b_entry.state = QueryValidatorEntryState::join(a_entry.state, b_entry.state);
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

impl fmt::Display for QueryValidator {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut has_errors = false;
        for entry in &self.entries {
            if (entry.state.is_error()) {
                if (! has_errors) {
                    write!(f, "Query would violate the borrow checker rules:")?;
                }
                has_errors = true;
            }
            match (entry.state) {
                QueryValidatorEntryState::MutableError => {
                    #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
                    // SAFETY: `entry.name` is a value previously generated using `core::any::type_name`.
                    { write!(f, "\n  Already mutably borrowed {}", unsafe{ UnqualifiedTypeName::from_unchecked(entry.name) })?; }
                    #[cfg(not(any(debug_assertions, feature = "keep_debug_names")))]
                    { write!(f, "\n  Already mutably borrowed item"); }
                },
                QueryValidatorEntryState::OwnedError => {
                    if (! has_errors) {
                        write!(f, "Query would violate the borrow checker rules:")?;
                    }
                    has_errors = true;
                    #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
                    // SAFETY: `entry.name` is a value previously generated using `core::any::type_name`.
                    { write!(f, "\n  Already took ownership of {}", unsafe{ UnqualifiedTypeName::from_unchecked(entry.name) })?; }
                    #[cfg(not(any(debug_assertions, feature = "keep_debug_names")))]
                    { write!(f, "\n  Already took ownership of item"); }
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

impl QueryValidatorEntry {

    /// Creates a new [`QueryValidatorEntry`] from a type `T` and [`QueryValidatorEntryState`].
    fn of<T : 'static>(
        state : QueryValidatorEntryState
    ) -> Self { QueryValidatorEntry {
        id    : TypeId::of::<T>(),
        #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
        name  : type_name::<T>(),
        state
    } }

}

impl QueryValidatorEntryState {

    /// Joins two [`QueryValidatorEntryState`]s together, or switches to an error if they conflict.
    fn join(a : Self, b : Self) -> Self {
        match ((a, b)) {
            ( Self::OwnedError   , _ ) | ( _ , Self::OwnedError   ) => Self::OwnedError,
            ( Self::Owned        , _ ) | ( _ , Self::Owned        ) => Self::OwnedError,
            ( Self::MutableError , _ ) | ( _ , Self::MutableError ) => Self::MutableError,
            ( Self::Mutable      , _ ) | ( _ , Self::Mutable      ) => Self::MutableError,
            ( Self::Immutable , Self::Immutable ) => Self::Immutable
        }
    }

    fn is_error(&self) -> bool {
        match (self) {
            QueryValidatorEntryState::Immutable    => false,
            QueryValidatorEntryState::Mutable      => false,
            QueryValidatorEntryState::Owned        => false,
            QueryValidatorEntryState::MutableError => true,
            QueryValidatorEntryState::OwnedError   => true
        }
    }

}
