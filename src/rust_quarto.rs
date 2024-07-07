// SPDX-License-Identifier: GPL-2.0
//! Rust Quarto game for Linux kernel

use kernel::prelude::*;

module! {
    type: QuartoModule,
    name: "quarto",
    author: "Chrysh",
    description: "My Rust kernel quarto game module",
    license: "GPL",
}

struct QuartoModule;

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
