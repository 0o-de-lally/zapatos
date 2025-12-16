#[cfg(test)]
mod find_enum {
    use move_core_types::language_storage::TypeTag;
    
    #[test]
    fn test_type_tag_variants() {
        // TypeTag should have these variants based on move-core-types
        // Let's see how many it actually has
        let test_bytes = vec![69u8]; // The problematic value
        let result: Result<TypeTag, _> = bcs::from_bytes(&test_bytes);
        match result {
            Ok(tag) => println!("Unexpectedly succeeded: {:?}", tag),
            Err(e) => println!("Error deserializing TypeTag with value 69: {:?}", e),
        }
        
        // Try with value 0-20 to see the range
        for i in 0..25u8 {
            let test_bytes = vec![i];
            let result: Result<TypeTag, _> = bcs::from_bytes(&test_bytes);
            match result {
                Ok(_) => println!("Value {} is valid for TypeTag", i),
                Err(_) => println!("Value {} is INVALID for TypeTag", i),
            }
        }
    }
}
