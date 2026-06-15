use anyhow::{Context, Result};
use colored::*;
use inquire::{Select, ui::{RenderConfig, Attributes, Color, StyleSheet}};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Deserialize)]
struct PackageJson {
    name: Option<String>,
    scripts: Option<HashMap<String, String>>,
}

fn find_package_json(start: &Path) -> Option<(PathBuf, PackageJson)> {
    let mut dir = start.to_path_buf();
    loop {
        let pkg_path = dir.join("package.json");
        if pkg_path.exists() {
            if let Ok(content) = fs::read_to_string(&pkg_path) {
                if let Ok(pkg) = serde_json::from_str::<PackageJson>(&content) {
                    if pkg.scripts.as_ref().map_or(false, |s| !s.is_empty()) {
                        return Some((pkg_path, pkg));
                    }
                }
            }
        }
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => break,
        }
    }
    None
}

fn detect_package_manager(dir: &Path) -> &'static str {
    if dir.join("bun.lockb").exists() {
        "bun"
    } else if dir.join("pnpm-lock.yaml").exists() {
        "pnpm"
    } else if dir.join("yarn.lock").exists() {
        "yarn"
    } else {
        "npm"
    }
}

fn run_script(pm: &str, script: &str) -> Result<()> {
    let status = Command::new(pm)
        .args(["run", script])
        .status()
        .with_context(|| format!("{pm} の実行に失敗しました"))?;

    std::process::exit(status.code().unwrap_or(1));
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cwd = std::env::current_dir()?;

    let (pkg_path, pkg) = find_package_json(&cwd)
        .context("❌ package.json が見つかりません")?;

    let pkg_dir = pkg_path.parent().unwrap();
    let pm = detect_package_manager(pkg_dir);
    let scripts = pkg.scripts.unwrap();

    // スクリプト名を安定した順序でソート
    let mut entries: Vec<(String, String)> = scripts.into_iter().collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    // 引数に直接スクリプト名が渡された場合
    if let Some(script_name) = args.first() {
        if entries.iter().any(|(name, _)| name == script_name) {
            println!(
                "\n{} {} run {}\n",
                "▶".green().bold(),
                pm,
                script_name.green().bold()
            );
            return run_script(pm, script_name);
        } else {
            eprintln!("{} スクリプト '{}' が見つかりません", "✖".red(), script_name);
            std::process::exit(1);
        }
    }

    // インタラクティブモード
    let project_name = pkg.name
        .as_deref()
        .unwrap_or("(no name)");

    println!(
        "\n{}  {} {}",
        "📦".bold(),
        project_name.cyan().bold(),
        format!("[{pm}]").dimmed()
    );

    // 表示用ラベル（スクリプト名 + コマンド）
    let max_name_len = entries.iter().map(|(n, _)| n.len()).max().unwrap_or(0);
    let labels: Vec<String> = entries
        .iter()
        .map(|(name, cmd)| {
            format!(
                "{}  {}",
                format!("{:<width$}", name, width = max_name_len).green().bold(),
                cmd.dimmed()
            )
        })
        .collect();

    let render_config = RenderConfig::default()
        .with_highlighted_option_prefix(inquire::ui::Styled::new("❯").with_fg(Color::LightGreen))
        .with_selected_option(Some(StyleSheet::new().with_attr(Attributes::BOLD)));

    let chosen_label = Select::new("どのスクリプトを実行する？", labels.clone())
        .with_render_config(render_config)
        .with_page_size(15)
        .prompt()
        .map_err(|_| {
            println!("{}", "キャンセルしました".dimmed());
            std::process::exit(0);
        })
        .unwrap();

    // ラベルからスクリプト名を逆引き
    let idx = labels.iter().position(|l| l == &chosen_label).unwrap();
    let (script_name, _) = &entries[idx];

    println!(
        "\n{} {} run {}\n",
        "▶".green().bold(),
        pm,
        script_name.green().bold()
    );

    run_script(pm, script_name)
}

