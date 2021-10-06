// Copyright (c) 2019 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// Error type returned by every parser on failure.
pub mod error;

/// Various helpers.
pub(crate) mod helpers;

/// Helper macros to parse and serialise more easily.
#[macro_use]
mod macros;
