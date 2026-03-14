// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! End-to-end tests for rhizoCrypt.
//!
//! These tests verify complete workflows from session creation through
//! dehydration, exercising the full RPC interface.

pub mod dag_operations;
pub mod dehydration_complete;
pub mod merkle_operations;
pub mod session_lifecycle;
pub mod slice_workflows;
