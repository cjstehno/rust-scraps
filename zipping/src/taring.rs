
#[allow(dead_code)]
pub fn foo() -> u32 {
    42
}

#[cfg(test)]
mod tests {
    use crate::taring::foo;
    use hamcrest2::prelude::*;
    use hamcrest2::equal_to;

    #[test]
    fn testing_it(){
        assert_that!(foo(), equal_to(42));
    }
}