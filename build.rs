use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;

fn main() -> Result<()> {
    // 这一行告诉 cargo 如果 /assets/ 目录中的内容发生了变化，就重新运行脚本
    println!("cargo:rerun-if-changed=assets/*");

    let out_dir = env::var("OUT_DIR")?;
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let mut paths_to_copy = Vec::new();
    paths_to_copy.push("assets/");
    copy_items(&paths_to_copy, out_dir, &copy_options)?;

    Ok(())
}