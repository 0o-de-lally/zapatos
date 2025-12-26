// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

//! Error types for IBE (Identity-Based Encryption) operations.

use anyhow::Error;

/// Type alias for IBE results using anyhow::Error for flexibility.
/// TODO: Consider using a dedicated IbeError enum if more structured error handling is needed.
pub type Result<T> = std::result::Result<T, Error>;
