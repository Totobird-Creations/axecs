//! TODO: Doc comments


/*
/// TODO: Doc comments
pub(crate) macro variadic {

    ( $( #[ $meta:meta ] )* $builder:ident ) => {
        variadic!{ $( #[ $meta ] )* $builder => T19, T18, T17, T16, T15, T14, T13, T12, T11, T10, T9, T8, T7, T6, T5, T4, T3, T2, T1, T0 }
    },

    ( $( #[ $meta:meta ] )* $builder:ident => $( $generic:ident ),* $(,)? ) => {
        variadic!{@inner ${count( $generic )} $( #[ $meta ] )* $builder => $( $generic ),* }
    },

    (@inner $count:tt $( #[ $meta:meta ] )* $builder:ident => $(,)? ) => {
        $builder!{
            #[doc(hidden)]
        }
    },

    (@inner $count:tt $( #[ $meta:meta ] )* $builder:ident => $first:ident $(,)? ) => {
        $builder!{
            $( #[ $meta ] )*
            #[doc = concat!("This trait is implemented for tuples up to ", stringify!($count), " items long.")]
            $first
        }
        variadic!{@inner $count $( #[ $meta ] )* $builder => }
    },

    (@inner $count:tt $( #[ $meta:meta ] )* $builder:ident => $first:ident $( , $next:ident )* $(,)? ) => {
        $builder!{
            #[doc(hidden)]
            $first $( , $next )*
        }
        variadic!{@inner $count $( #[ $meta ] )* $builder => $( $next ),* }
    }

}*/


/// TODO: Doc comments
pub(crate) macro variadic_no_unit {

    ( $( #[ $meta:meta ] )* $builder:ident ) => {
        variadic_no_unit!{ $( #[ $meta ] )* $builder => T19, T18, T17, T16, T15, T14, T13, T12, T11, T10, T9, T8, T7, T6, T5, T4, T3, T2, T1, T0 }
    },

    ( $( #[ $meta:meta ] )* $builder:ident => $( $generic:ident ),* $(,)? ) => {
        variadic_no_unit!{@inner ${count( $generic )} $( #[ $meta ] )* $builder => $( $generic ),* }
    },

    (@inner $count:tt $( #[ $meta:meta ] )* $builder:ident => $(,)? ) => { },

    (@inner $count:tt $( #[ $meta:meta ] )* $builder:ident => $first:ident $(,)? ) => {
        $builder!{
            $( #[ $meta ] )*
            #[doc = concat!("This trait is implemented for tuples up to ", stringify!($count), " items long.")]
            $first
        }
        variadic_no_unit!{@inner $count $( #[ $meta ] )* $builder => }
    },

    (@inner $count:tt $( #[ $meta:meta ] )* $builder:ident => $first:ident $( , $next:ident )* $(,)? ) => {
        $builder!{
            #[doc(hidden)]
            $first $( , $next )*
        }
        variadic_no_unit!{@inner $count $( #[ $meta ] )* $builder => $( $next ),* }
    }

}
