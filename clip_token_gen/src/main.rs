use rand::Rng;

/// Token configuration
const TOKEN_LENGTH_BYTES: usize = 32; // 256 bits for security

fn main() {
    println!("🔐 ClipShare Token Generator");
    println!("===========================\n");

    // Generate a cryptographically secure random token
    let token = generate_secure_token();

    println!("✅ Token generated successfully!");
    println!("\n📋 Your authentication token:");
    println!("{}\n", token);

    println!("📝 Usage instructions:");
    println!("1. Set the environment variable on your server:");
    println!("   export CLIPSHARE_TOKEN=\"{}\"", token);
    println!("   # Or on Windows PowerShell:");
    println!("   $env:CLIPSHARE_TOKEN=\"{}\"", token);
    println!("\n2. Set the same environment variable on your client:");
    println!("   export CLIPSHARE_TOKEN=\"{}\"", token);
    println!("   # Or on Windows PowerShell:");
    println!("   $env:CLIPSHARE_TOKEN=\"{}\"", token);
    println!("\n3. For persistent configuration, add to your shell profile (~/.bashrc, ~/.zshrc, etc.)");

    println!("\n⚠️  Security notes:");
    println!("   - Keep this token secret and secure");
    println!("   - Don't share it publicly or commit it to version control");
    println!("   - Generate a new token if you suspect it has been compromised");
    println!("   - Use different tokens for different environments (dev/prod)");
}

/// Generates a cryptographically secure random token
fn generate_secure_token() -> String {
    use base64::prelude::*;

    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; TOKEN_LENGTH_BYTES];
    rng.fill(&mut bytes[..]);

    // Encode as base64 for easy transmission
    BASE64_STANDARD.encode(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation() {
        let token1 = generate_secure_token();
        let token2 = generate_secure_token();

        // Tokens should be different each time
        assert_ne!(token1, token2);

        // Tokens should be non-empty
        assert!(!token1.is_empty());
        assert!(!token2.is_empty());

        // Tokens should be base64 encoded (only contain valid characters)
        assert!(token1.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '='));
    }

    #[test]
    fn test_token_length() {
        let token = generate_secure_token();
        // Base64 encoding of 32 bytes should produce 44 characters (with padding)
        assert_eq!(token.len(), 44);
    }
}
