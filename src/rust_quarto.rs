// SPDX-License-Identifier: GPL-2.0
//! Rust Quarto game for Linux kernel

use kernel::prelude::*;

#[macro_use]
pub mod vec_extra;
mod hashset;

use core::hash::{Hash, Hasher};
use crate::hashset::HashSet;
// Needed for macro vec!
use crate::vec_extra::VecExtra;

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

        let remaining_pieces_vec = vec![ 1 ,2, 4 ];
        let remaining_pieces = remaining_pieces_vec
           .iter()
           .copied()
           .collect::<HashSet<_>>();
        let mut foobar = remaining_pieces.clone();
        foobar.remove(&2);

        Ok(QuartoModule)
    }
}

impl Drop for QuartoModule {
    fn drop(&mut self) {
        pr_info!("Bye-bye from Quarto Rust module!\n");
    }
}
