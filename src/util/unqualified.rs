//! Utilities for reducing full type paths to just the type names.


use core::fmt;


/// Lazily reduces a fully qualified type name to just the type names.
#[derive(Clone, Copy)]
pub(crate) struct UnqualifiedTypeName {

    /// The fully qualified type name.
    fully_qualified_type_name : &'static str

}

impl UnqualifiedTypeName {

    /// Wraps a fully qualified type name.
    ///
    /// # Safety:
    /// The caller is responsible for ensuring that the given type name is valid.
    pub(crate) unsafe fn from_unchecked(fully_qualified_type_name : &'static str) ->  Self { Self {
        fully_qualified_type_name
    } }

}

impl fmt::Debug for UnqualifiedTypeName {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self)
    }
}

impl fmt::Display for UnqualifiedTypeName {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut chars      = self.fully_qualified_type_name.chars().enumerate().peekable();
        let mut word_start = 0;
        let mut was_close  = false;
        loop {
            let Some((i, ch)) = chars.next() else { break };

            if (ch == ':' && chars.peek().is_some_and(|(_, ch)| *ch == ':')) {
                let _ = chars.next();
                word_start = i + 2;
                if (was_close) {
                    write!(f, "::")?;
                }
            }

            else if (is_close_char(ch)) {
                write!(f, "{}{}", &self.fully_qualified_type_name[word_start..i], ch)?;
                word_start = i + 1;
                was_close = true;
                continue;
            }

            else if (is_split_char(ch)) {
                write!(f, "{}{}", &self.fully_qualified_type_name[word_start..i], ch)?;
                word_start = i + 1;
            }

            was_close = false;
        }
        write!(f, "{}", &self.fully_qualified_type_name[word_start..])?;
        Ok(())
    }
}


/// Returns `true` if the character terminates a single type path.
fn is_split_char(ch : char) -> bool {
    ch == ' ' || ch == '<' || ch == '>' || ch == '(' || ch == ')' || ch == '[' || ch == ']' || ch == ',' || ch == ';'
}

/// Returns `true` if the character closes a group.
fn is_close_char(ch : char) -> bool {
    ch == '>' || ch == ')' || ch == ']'
}



#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    #[test]
    fn unqualify_type_name() {
        assert_eq!(
            format!("{}", unsafe{ UnqualifiedTypeName::from_unchecked("alloc::string::String") }),
            "String"
        );
        assert_eq!(
            format!("{}", unsafe{ UnqualifiedTypeName::from_unchecked("core::option::Option<(std::vec::Vec<*mut u8>,)>") }),
            "Option<(Vec<*mut u8>,)>"
        );
        assert_eq!(
            format!("{}", unsafe{ UnqualifiedTypeName::from_unchecked("<axecs::tests::SomeType as axecs::tests::SomeTrait>::SomeTraitAssociatedType") }),
            "<SomeType as SomeTrait>::SomeTraitAssociatedType"
        );
    }

}
