fn main() {
    println!("Hello, rust!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        main();
    }
}
