
/// Generates a random sessionid.
pub fn generate_sessionid() -> String {
    // Should look like "37bf523a24034ec06c60ec61"
    (0..12)
        .map(|_| { 
            let b = rand::random::<u8>();
            
            format!("{b:02x?}")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn generates_session() {
        let sessionid = generate_sessionid();
        
        assert_eq!(sessionid.len(), 24);
    }
}