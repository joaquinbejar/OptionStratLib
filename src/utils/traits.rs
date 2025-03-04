/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/2/25
******************************************************************************/

/// Trait to get the length of an object.
pub trait Len {
    /// Returns the length of the object.
    fn len(&self) -> usize;

    /// Returns `true` if the object is empty, `false` otherwise.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}