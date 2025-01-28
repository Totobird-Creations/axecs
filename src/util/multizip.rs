//! TODO: Doc commends


/// Zips multiple [`Iterator`]s into a single [`Iterator`] over a set of tuples containing a single entry from each given [`Iterator`].
///
/// This is a variadic version of [`Iterator::zip`].
pub(crate) macro multizip {

    ( $( $generic:ident ),+ $(,)? ) => { {

        struct Zip< $( $generic : Iterator , )* >( $( $generic , )* );

        #[allow(non_snake_case)]
        impl< $( $generic : Iterator , )* > Iterator for Zip< $( $generic , )* > {
            type Item = ( $( $generic::Item , )* );
            fn next(&mut self) -> Option<Self::Item> {
                let Self( $( $generic , )* ) = self;
                Some(( $( $generic.next()? , )* ))
            }
        }

        Zip ( $( $generic , )* )

    } }

}
