use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use rock_dom::prelude::*;
use rock_layout::prelude::*;
use rock_render::{prelude::*, text::TextSystem};
use std::{error::Error, fs::File, io::Read, path::PathBuf};
use tracing::trace;
use tracing_subscriber::FmtSubscriber;

use crate::app::{Renderer, Run};

mod app;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to a HTML file
    path: PathBuf,

    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

struct UnoptimizedRenderer {
    dom: Dom,
    ts: TextSystem,
}

impl Renderer for UnoptimizedRenderer {
    fn draw(&mut self, width: u32, height: u32, scale: f32) -> tiny_skia::Pixmap {
        let viewport = Size {
            w: width as f32 / scale,
            h: height as f32 / scale,
        };
        let boxes = build_box_tree(&self.dom).unwrap(); // Box node of root
        let root = layout(
            &boxes,
            Constraints {
                available_width: viewport.w,
            },
            &mut self.ts,
        );
        let list = build_display_list(&root, viewport);
        render(&list, width, height, scale, &mut self.ts)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    FmtSubscriber::builder().with_max_level(args.verbose).init();

    let mut buf = String::new();

    File::open(args.path)?.read_to_string(&mut buf)?;
    trace!("value: {buf}");

    // just weird makeshift code for now
    let dom = create_dom(&buf)?;

    UnoptimizedRenderer {
        dom,
        ts: Default::default(),
    }
    .run_simple();

    Ok(())
}
