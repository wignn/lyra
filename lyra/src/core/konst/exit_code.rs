/// A harmless notice, confirming something the user might have meant to do.
pub const NOTICE: &str = "[Notice]";

/// A suspicious notice, implying something the user might not have meant to do.
pub const DUBIOUS: &str = "[?]";

/// A harmless warning.
pub const WARNING: &str = "[Warning]";

/// Needed information was not found, implying user given an incorrect query.
#[allow(unused)]
pub const NOT_FOUND: &str = "[Not Found]";

/// Invalid command usage, implying unmet conditions.
pub const INVALID: &str = "[Invalid]";

/// User lacked sufficient permissions.
pub const PROHIBITED: &str = "[Prohibited]";

/// Bot lacked sufficient permissions.
pub const FORBIDDEN: &str = "[Forbidden]";

/// Other known errors.
pub const KNOWN_ERROR: &str = "[Error]";

/// Unknown errors.
pub const UNKNOWN_ERROR: &str = "[Error]";
