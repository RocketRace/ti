//! # `ti`
//!
//! `ti` is a tiny terminal-based pixel graphics engine.
//!
//! `ti` renders using unicode Braille characters. In addition to raw pixel output,
//! it supports writing ANSI terminal colors and sprite drawing.
pub mod cell;
pub mod color;
pub mod screen;
pub mod sprite;
pub(crate) mod units;
