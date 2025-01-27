//! TODO: Doc commends


/// Zips multiple [`Iterator`]s into a single [`Iterator`] over a set of tuples containing a single entry from each given [`Iterator`].
///
/// This is a variadic version of [`Iterator::zip`].
pub(crate) macro multizip {

    ( $( $generics:ident ),+ $(,)? ) => { {

        struct Zip< $( $generics : Iterator , )* >( $( $generics , )* );

        impl< $( $generics : Iterator , )* > Iterator for Zip< $( $generics , )* > {
            type Item = ( $( $generics::Item , )* );
            fn next(&mut self) -> Option<Self::Item> {
                #[allow(non_snake_case)]
                let Self( $( $generics , )* ) = self;
                Some(( $( $generics.next()? , )* ))
            }
        }

        Zip ( $( $generics , )* )

    } }

}
