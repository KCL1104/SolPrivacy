use colored::Colorize;

/// Decodes Solana custom program errors into human-readable messages
pub fn decode_transaction_error(error_string: &str) -> String {
    // Check for common error patterns
    if let Some(code) = extract_custom_error_code(error_string) {
        if let Some(msg) = map_token_error(code) {
            return format!("{} ({})\n  {}", "Transaction Failed".red().bold(), error_string, msg.yellow());
        }
    }
    
    // Fallback
    format!("{} ({})", "Transaction Failed".red().bold(), error_string)
}

fn extract_custom_error_code(err: &str) -> Option<u32> {
    // Format: "custom program error: 0x1"
    if let Some(start) = err.find("custom program error: 0x") {
        let remainder = &err[start + 22..];
        let end = remainder.find(|c: char| !c.is_ascii_hexdigit()).unwrap_or(remainder.len());
        let hex_str = &remainder[..end];
        return u32::from_str_radix(hex_str, 16).ok();
    }
    None
}

/// Maps SPL Token / Token-2022 error codes
fn map_token_error(code: u32) -> Option<&'static str> {
    // Reference: https://github.com/solana-labs/solana-program-library/blob/master/token/program/src/error.rs
    match code {
        0x0 => Some("Lamport balance below rent-exempt threshold. (The account needs more SOL)"),
        0x1 => Some("Insufficient funds. (You don't have enough tokens/SOL)"),
        0x2 => Some("Invalid Mint."),
        0x3 => Some("Account not associated with this Mint."),
        0x4 => Some("Owner does not match."),
        0x5 => Some("Fixed Supply. (Cannot mint to a fixed supply mint)"),
        0x6 => Some("Already in use."),
        0x7 => Some("Invalid number of provided signers."),
        0x8 => Some("Invalid signer."),
        0x9 => Some("Invalid argument."),
        0xA => Some("Invalid instruction."),
        0xB => Some("State is invalid for requested operation."),
        0xC => Some("Overflow."),
        0xD => Some("Authority Type not supported."),
        0xE => Some("Mint decimals mismatch."),
        0xF => Some("Non-native account can only send to non-native account."),
        _ => None,
    }
}
