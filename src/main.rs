#![forbid(unsafe_code)]

use std::cell::RefCell;
use crate::minecraft::{Fetch, Resources, VersionInfo};
use scopeguard::defer;

mod cache;
mod checksum;
mod config;
mod db;
mod install_operation;
mod minecraft;

use crate::db::{Installation, JsonFileDb};
use clap::{App, Arg, SubCommand};
use config::Config;
use futures::future::try_join_all;

fn prepare_config() -> anyhow::Result<Config> {
    let config = Config::new();
    config.prepare_dirs()?;
    Ok(config)
}

fn build_cli() -> App<'static> {
    App::new("mc-installer")
        .version("0.1")
        .author("Joshua Locash <locashjosh@gmail.com>")
        .subcommand_required(true)
        .arg(
            Arg::with_name("db_path")
                .short('d')
                .long("db-path")
                .help("override default database path"),
        )
        .subcommand(
            SubCommand::with_name("install")
                .about("installs a new version of Minecraft")
                .arg(
                    Arg::with_name("version")
                        .required(true)
                        .help("version name (e.g '1.12')")
                        .index(1),
                ),
        )
        .subcommand(SubCommand::with_name("list").about("lists installed versions"))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = prepare_config()?;
    let db = RefCell::new(JsonFileDb::open(&config.db_path)?);
    println!("I will look for cached data in {:?}", &config.cache_path);

    let manifest = cache::read_manifest(&config).await?;
    defer! {
        cache::write_manifest(&config, &manifest).unwrap_or_else(|err| panic!("error writing version manifest ({})", err));
        db.borrow_mut().commit().unwrap_or_else(|err| panic!("error committing database ({})", err))
    }

    let matches = build_cli().get_matches();
    match matches.subcommand() {
        Some(("install", install_matches)) => {
            let version_str = install_matches.get_one::<String>("version").unwrap();
            let version_info = manifest
                .versions
                .iter()
                .find(|info| &info.id == version_str);
            if let Some(vinfo) = version_info {
                db.borrow_mut().db.installations.push(install(&config, vinfo).await?);
            } else {
                println!("version {} not found!", version_str)
            }
        }
        _ => unreachable!("Subcommands are required!"),
    }

    Ok(())
}

async fn install(config: &Config, version_info: &VersionInfo) -> anyhow::Result<Installation> {
    println!("Fetching version info...");
    let v = version_info.fetch_async().await?;

    println!("Fetching asset index...");
    let asset_index = v.asset_index.fetch_async().await?;

    let inst = Installation::new(&v, &config.db_path.join("installations"));
    inst.ensure_dirs_exist()?;

    std::fs::write(
        inst.path.join(format!("{}.json", v.id)),
        serde_json::to_vec(&v)?.as_slice(),
    )?;

    std::fs::write(
        inst.path.join("asset_index.json"),
        serde_json::to_vec(&asset_index)?.as_slice(),
    )?;

    println!("Installing libraries");
    try_join_all(
        v.resources()
            .iter()
            .map(|lib| lib.install_to(&inst.lib_dir)),
    )
    .await?;

    println!("Installing assets");
    try_join_all(
        asset_index
            .resources()
            .iter()
            .map(|asset| asset.install_to(&inst.path)),
    )
    .await?;
    Ok(inst)
}
