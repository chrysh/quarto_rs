// SPDX-License-Identifier: GPL-2.0
//! Rust Quarto game for Linux kernel

use kernel::prelude::*;
use alloc::vec::Vec;

use core::hash::{Hash, Hasher};
use self::hashset::HashSet;

mod hashset;

module! {
    type: QuartoModule,
    name: "quarto",
    author: "Chrysh",
    description: "My Rust kernel quarto game module",
    license: "GPL",
}

struct QuartoModule;

pub struct Piece {
    pub properties: u32,
}

impl Hash for Piece {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.properties.hash(state);
    }
}

impl kernel::Module for QuartoModule {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Loading Quarto Rust module\n");
        Ok(QuartoModule)
    }
}

impl Drop for QuartoModule {
    fn drop(&mut self) {
        pr_info!("Bye-bye from Quarto Rust module!\n");
    }
}
