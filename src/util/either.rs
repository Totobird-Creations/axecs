//! A value that can be one of two things.


/// A value that can be one of two things.
pub(crate) enum Either<A, B> {

    /// The value is of type `A`.
    A(A),

    /// The value is of type `B`.
    B(B)

}
