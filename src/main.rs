#![no_std] // Use this attribute to indicate no_std environment
/*!
Welcome to `quarto_rs`
*/

#![warn(clippy::cargo)]
#![deny(clippy::cargo_common_metadata)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(unsafe_code)]
#![allow(
    clippy::unreadable_literal,
    clippy::type_repetition_in_bounds,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::missing_docs_in_private_items
)]
#![deny(
    missing_debug_implementations,
    missing_docs,
    //trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_must_use,
    missing_docs,
    //unused_results
)]
#![cfg_attr(
    not(debug_assertions),
    deny(
        bad_style,
        improper_ctypes,
        non_shorthand_field_patterns,
        no_mangle_generic_items,
        overflowing_literals,
        path_statements,
        patterns_in_fns_without_body,
        private_in_public,
        unconditional_recursion,
        unused_allocation,
        unused_comparisons,
        unused_parens,
        while_true
    )
)]

mod ai;
mod field;
mod game;
mod piece;
mod rng;
pub mod hashset;
mod vec_extra;
mod rust_quarto;

// TODO use std::{env::args, io::stdin};

use game::ArrayBase;

use crate::{
    ai::SimpleAi,
    field::{try_parse_pos, Field},
    game::{Game, Player, Status},
    piece::Piece,
    rng::{time_nanos, RomuDuoJrRand},
};

fn main() {
    if args().any(|x| x.contains("help") || x == "-h") {
        // Orig: let current_exe = std::env::current_exe().unwrap();
        // Orig: let current_exe_name = current_exe.file_name().unwrap().to_string_lossy();
        let current_exe_name = "quarto_rs";
        pr_info!(
            "Your friendly Quarto game.

The game is played on a 4x4 board with 16 pieces. Each piece has four distinct
characteristics: size (large/✋ or small/🤏), color (light/⬜ or dark/⬛),
shape (round/🟠 or square/🔶), and fill (filled/🔴 or hollow/⭕).
On your turn, you choose one of the 16 pieces and give it to your opponent.
Your opponent then places that piece on any empty space on the board.

The first player to create a row of four pieces with at least one matching
characteristic wins the game. Matching characteristics can be in any direction,
horizontally, vertically, or diagonally.
In the harder square mode, (-q), a square of 4 is also considered a win.

If all 16 pieces have been placed and there is no winner, the game is a tie.

Usage: {current_exe_name} <Options>

Options:
    --square-mode|-q:   Enable harder rules: not only 4 of the same in a row,
                        but also a square of 4 is considered a win.
    --base0|-0:         Starts to count at 0 instead of 1 (programmer style)
    --ai-reasoning|-r:  Print information about what the AI is doing, and why,
                        during the game.
    --ai-simulation|-a: Simulate a bunch of AI battles.
    --seed=<>|-s=<>:    Seed the AI RNG
    --pvp|-p            No AI, just humans (player vs player)
    --help|-h:          Print this help screen.

Good luck!
"
        );
        return;
    }

    let mut game = Game::new(Player::PlayerOne);

    if args().any(|x| x == "--ai-reasoning" || x == "-r") {
        game.ai_reasoning = true;
    }

    if let Some(seed) = args().find(|x| x.starts_with("--seed") || x.starts_with("-s=")) {
        let mut seed = seed.split('=');
        let _ = seed.next();
        let seed_str = seed.next().unwrap();
        let Ok(seed) = seed_str.parse() else {
            pr_info!("Invalid seed: {seed_str}");
            return;
        };
        game.seed = Some(seed);
    }

    if args().any(|x| x == "--square-mode" || x == "-q") {
        game.field.square_mode = true;
    }

    if args().any(|x| x == "--base0" || x == "-0") {
        game.array_base = ArrayBase::Zero;
    }

    if args().any(|x| x == "--pvp" || x == "-p") {
        game.pvp = true;
    }

    if args().any(|x| x == "--ai-simulation" || x == "-a") {
        if game.pvp {
            pr_info!("PvP mode and ai-simulation don't match.. :)");
        } else {
            ai_simulation(&game);
        }
        return;
    }

    play(game);
}

fn play(mut game: Game) {
    let mut buf = String::new();
    #[allow(clippy::cast_possible_truncation)]
    let seed = game.seed.unwrap_or_else(|| time_nanos() as u64);

    pr_info!("Game Seed: {seed}");

    let human = RomuDuoJrRand::with_seed(seed).choose([Player::PlayerOne, Player::PlayerTwo]);
    let mut ai = SimpleAi::with_seed(human.next(), seed);

    if !game.pvp {
        pr_info!("You are {human}.");
    }

    pr_info!();
    pr_info!("Let the games begin!");

    loop {
        game.pp();
        if !game.running() {
            return;
        }

        if game.pvp || game.player() == human {
            if game.is_initial_move() {
                let next_piece = read_piece(&game);
                game.initial_move(next_piece).unwrap();
            } else {
                loop {
                    pr_info!("Select x,y to put the piece to:");
                    buf.clear();
                    // TODO: stdin().read_line(&mut buf).unwrap();
                    buf = "1,1"
                    let base = game.array_base;
                    let pos = try_parse_pos(&buf).map(|(x, y)| (base.unbased(x), base.unbased(y)));
                    if let Ok(pos) = pos {
                        if pos.0 < Field::SIZE && pos.1 < Field::SIZE {
                            let next_piece = read_piece(&game);
                            if game.do_move(pos, next_piece).is_ok() {
                                break;
                            }
                        }
                    }
                    pr_info!("Illegal move! The x,y value must be an empty place on the field!");
                    pr_info!();
                }
            }
            pr_info!();
        } else {
            game = ai.play_iteratively(&mut game);
        }
    }
}

fn read_piece(game: &Game) -> Piece {
    let mut buf = String::with_capacity(16);
    let base = game.array_base;
    let piece_id: usize = loop {
        pr_info!(
            "\n{}, please chose your opponent's next piece ({}-{}):",
            game.player(),
            base.based(0),
            base.based(game.remaining_pieces().len() - 1),
        );
        buf.clear();
        // TODO stdin().read_line(&mut buf).unwrap();
        buf = "1"
        let num = buf.trim().parse().map(|x| base.unbased(x));
        if let Ok(num) = num {
            if num < game.remaining_pieces().len() {
                break num;
            }
        }
        let buf = buf.strip_suffix('\n').unwrap();
        #[cfg(debug_assertions)]
        pr_info!("{:?} (str: '{buf}')", num.err());
        pr_info!("Illegal choice: '{buf}', please pick the id of a remaining piece:");
        game.pp_remaining_pieces();
    };
    game.remaining_pieces()[piece_id]
}

#[allow(clippy::cast_precision_loss, clippy::cast_lossless)]
fn ai_simulation(base_game: &Game) {
    const ITERS: usize = 100;

    // TODO let it = std::time::Instant::now();

    let mut ai_one_wins = 0;
    let mut ai_two_wins = 0;
    let mut turns = 0_u64;

    #[allow(clippy::cast_possible_truncation)]
    let seed = base_game.seed.unwrap_or_else(|| time_nanos() as u64);
    let mut rng = RomuDuoJrRand::with_seed(seed);

    pr_info!("Using seed {seed}");

    'outer: for _ in 0..ITERS {
        let mut game = base_game.clone();

        let mut ai_one = SimpleAi::with_seed(Player::PlayerOne, rng.next());
        let mut ai_two = SimpleAi::with_seed(Player::PlayerTwo, rng.next());

        loop {
            if base_game.ai_reasoning {
                game.pp();
            }
            if !game.running() {
                if let Status::Won { winner } = game.status {
                    if winner == Player::PlayerOne {
                        ai_one_wins += 1;
                    } else {
                        ai_two_wins += 1;
                    }
                }
                turns += game.round() as u64;
                continue 'outer;
            }

            if game.player() == Player::PlayerOne {
                game = ai_one.play_iteratively(&mut game);
            } else {
                game = ai_two.play_iteratively(&mut game);
            }
        }
    }

    // TODO let elapsed = it.elapsed();

    pr_info!(
        "Did {} games in {} seconds({:05.3} games/sec)",
        ITERS,
        elapsed.as_secs(),
        ITERS as f64 / (elapsed.as_secs() as f64)
    );
    pr_info!(
        "Did {} turns in total, average of {} turns per game",
        turns,
        turns as f64 / ITERS as f64
    );
    pr_info!(
        "Player 1 had {} wins({}%), Player 2 had {} wins({}%).",
        ai_one_wins,
        (ai_one_wins as f64 / ITERS as f64) * 100.,
        ai_two_wins,
        (ai_two_wins as f64 / ITERS as f64) * 100.
    );
    let draws = ITERS - ai_one_wins - ai_two_wins;
    let draw_percentage = (draws as f64 / ITERS as f64) * 100.;

    pr_info!("We had {draws} draws ({draw_percentage}%)");
}

#[cfg(test)]
mod test {
    use crate::{
        field::Field,
        game::ArrayBase,
        piece::{Piece, Property},
    };

    #[test]
    fn test_check_field_for_win() {
        let test_light_tall: Piece =
            Piece::with_props(Property::Tall as u8 | Property::Light as u8);
        let test_dark_short: Piece = Piece::with_props(0);

        let mut field = Field::new();

        field.put((3, 0), test_light_tall).unwrap();
        field.put((2, 1), test_light_tall).unwrap();
        field.put((1, 2), test_dark_short).unwrap();

        assert!(!field.check_field_for_win());

        field.put((0, 3), test_light_tall).unwrap();

        field.pp(ArrayBase::One);
    }
}
