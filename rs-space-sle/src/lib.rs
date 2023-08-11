//! # CCSDS SLE (Space Link Extension) Implementation
//!
//! This library provides an implementation of the CCSDS Space Link Extension protocol used
//! to communicate via space networks (e.g. ground stations or sometimes test equipment).
//!
//! This library does not aim to fully implement all features of the ESA SLE API, but is
//! intended to be locally usable. The ESA SLE API follows the Windows COM model and is
//! quite complicated to use. The goal of this reimplementation in Rust is to provide a
//! more simple approach, which is correct and also performant.
//!
//! Some features will not be present in this library:
//! - The ESA SLE API provides a communication server, where multiple SLE services can register and a proxy component is used for communication. This library dose not provide this, it provides direct TCP/IP sockets.
//! - Only a subset of services is actually implemented. This, however, can be extended
//!
//! The goal is also to provide a SLE Provider and a SLE User implementation, so that both
//! sides of the communication protocol can be used.
//!
//! This library was developed agains the ESA C++ SLE API as well as the ESA SLETT (SLE Test Tool).
//! Also, it is a kind of port of the Haskell library `esa-sle-native` which is used within the
//! *ParagonTT* test tool, in use at the German Space Agency (DLR GSOC) and ESA ESTEC in automated
//! test cases against the mission control systems there.

/// General types to be used
pub mod types {
    /// AUL is the authentication layer of the SLE messages and provides types for handling the 
    /// authentication
    pub mod aul;
    /// Contains general types and functions for ASN1 functionality
    pub mod sle;
}
/// TML is the TCP/IP message layer for SLE. All SLE PDUs are ASN1 encoded messages, which are
/// transmitted via the TML messages defined in this module
pub mod tml {
    pub mod config;
    pub mod message;
}
/// Provides the RAF (Return All Frames) telemetry service. User and Providers are present, as well
/// as the specific configs and ASN1 definitions specific for this service.
pub mod raf {
    pub mod asn1;
    pub mod user;
    pub mod config;
    pub mod provider;
    pub mod provider_state;
    pub mod state;
}
/// This module contains the general SLE configuration values.
pub mod sle {
    pub mod config;
}
/// This module contains the ASN1 definitions for the SLE PDUs
pub mod asn1;
/// Contains the configuration for the SLE User
pub mod user {
    pub mod config;
}
/// Contains the configuration and callback interfaces for the SLE Provider.
pub mod provider {
    pub mod app_interface;
    pub mod config;
}
