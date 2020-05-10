extern crate pretty_env_logger;
#[macro_use] extern crate log;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde;
use std::env;
use std::{io::Read, fs, fs::read_dir, path::PathBuf};
use std::process::exit;
use anvil_region::AnvilChunkProvider;
use clap::{Arg, App, SubCommand};

mod utils;
mod models;

use models::*;
use utils::copy;


pub struct World {
    pub path: PathBuf,
}

impl World {
    pub fn new(path: PathBuf) -> Self {
        Self {path}
    }

    pub fn region_path(&self, x: i32, z: i32) -> PathBuf {
        let mut path = self.path.clone();
        path.push(format!("r.{}.{}.mca", x, z));
        path
    }

    /// Copy the current world to another path and return the new world
    pub fn dup(self, path: PathBuf) -> std::io::Result<World> {
        copy(self.path, &path)?;
        Ok(Self {
            path
        })
    }

    /// Fill a rectangle of regions with `source_region`
    pub fn fill_copy(&self, xmin: i32, zmin: i32, xmax: i32, zmax: i32, source_region: PathBuf) -> std::io::Result<()> {
        for x in (xmin..xmax).into_iter() {
            for z in (zmin..zmax).into_iter() {
                let target = self.region_path(x, z);
                let _ = std::fs::remove_file(&target);
                let _ = std::fs::copy(&source_region, target);
            }
        }
        Ok(())
    }

    pub fn dup_region(&self, source: (i32, i32), dest: (i32, i32)) -> std::io::Result<()> {
        let _ = std::fs::remove_file(self.region_path(dest.0, dest.1));
        std::fs::copy(self.region_path(source.0, source.1), self.region_path(dest.0, dest.1))?;
        Ok(())
    }
}

fn run(input: &str, output: &str, patch: &str) -> std::io::Result<()> {
    let world = World::new(PathBuf::from(input))
        .dup(PathBuf::from(output))?;
    let src = world.region_path(0, 0);
    let chunk_provider = AnvilChunkProvider::new(world.path.to_str().unwrap());
    let reader = std::fs::read_dir(patch)?;
    let patch_chunk = chunk_provider.load_chunk(0,0).expect("Base chunk in source world (0,0)");
    for entry in reader {
        if let Some(entry) = entry.ok()
            .and_then(|entry| std::fs::OpenOptions::new().read(true).open(entry.path()).ok())
            .and_then(|mut file| {
                let mut st = String::new();
                file.read_to_string(&mut st).ok()?;
                Some(st)
            })
            .and_then(|e| serde_json::from_str(&e).ok())
        {
            let chunk: PacketChunk = entry;
            if !world.region_path(chunk.x, chunk.z).exists() {
                if let Err(e) = world.dup_region((0, 0), (chunk.x >> 5, chunk.z >> 5)) {
                    error!("Failed to dup reguion: {:?}", e);
                }
            }
            let chunk_x = chunk.x;
            let chunk_z = chunk.z;
            match chunk_provider.save_chunk(chunk_x, chunk_z, chunk.into()) {
                Ok(_) => info!("{}:{} Patched !", chunk_x, chunk_z),
                Err(e) => error!("{}:{} Failed to patch: {:?}", chunk_x, chunk_z, e),
            }
        }
    }
    Ok(())
}

fn main() {
    pretty_env_logger::init();
    let matches = App::new("dump-to-map")
        .arg(
            Arg::with_name("input")
                .help("A valide minecraft 1.15 world that is used as base")
                .short("i")
                .required(true)
                .takes_value(true)
        )
        .arg(
            Arg::with_name("patch")
                .help("A directory containing JOSN chunk dump generated by cort2bot")
                .short("p")
                .required(true)
                .takes_value(true)
        )
        .arg(
            Arg::with_name("output")
                .help("Output directory (the new world will be generated into)")
                .short("o")
                .required(true)
                .takes_value(true)
        )
        .get_matches();
    let output = matches.value_of("output").unwrap();
    let patch = matches.value_of("patch").unwrap();
    let input = matches.value_of("input").unwrap();
    if let Err(e) = run(input, output, patch) {
        eprintln!("{}", e);
    }
}
