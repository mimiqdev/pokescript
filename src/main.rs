use anyhow::{anyhow, Result};
use clap::{CommandFactory, Parser};
use colored::*;
use include_dir::{include_dir, Dir};
use rand::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

static POKEMON_JSON: &str = include_str!("../pokemon.json");
static COLORSCRIPTS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/colorscripts");

const SHINY_RATE: f64 = 1.0 / 128.0;

#[derive(Parser, Debug)]
#[command(
    name = "pokescript",
    author,
    version,
    about = "CLI utility to print out unicode image of a pokemon in your shell",
    override_usage = "pokescript [OPTION] [POKEMON NAME]",
    help_template = "{about-with-newline}\n{usage-heading} {usage}\n\n{all-args}{after-help}"
)]
struct Cli {
    #[arg(
        short,
        long,
        help = "Print list of all pokemon",
        action = clap::ArgAction::SetTrue
    )]
    list: bool,

    #[arg(short, long, help = "Select pokemon by name")]
    name: Option<String>,

    #[arg(short, long, help = "Show an alternate form of a pokemon")]
    form: Option<String>,

    #[arg(long, help = "Do not display pokemon name", action = clap::ArgAction::SetFalse)]
    show_title: bool,

    #[arg(short, long, help = "Show the shiny version of the pokemon instead")]
    shiny: bool,

    #[arg(
        short = 'b',
        long = "big",
        help = "Show a larger version of the sprite"
    )]
    large: bool,

    #[arg(
        short,
        long,
        help = "Show a random pokemon from a specific generation (1-8) or range (eg. 1-3)",
        num_args = 0..=1,
        default_missing_value = "1-8"
    )]
    random: Option<String>,

    #[arg(
        long = "random-by-names",
        help = "Show a random pokemon from a comma-separated list of names (eg. charmander,bulbasaur)"
    )]
    random_by_names: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Pokemon {
    name: String,
    forms: Vec<String>,
}

fn load_pokemon_data() -> Result<Vec<Pokemon>> {
    let pokemon_json: Vec<Pokemon> = serde_json::from_str(POKEMON_JSON)?;
    Ok(pokemon_json)
}

fn generations() -> HashMap<u8, (u32, u32)> {
    let mut gen_map = HashMap::new();
    gen_map.insert(1, (1, 151));
    gen_map.insert(2, (152, 251));
    gen_map.insert(3, (252, 386));
    gen_map.insert(4, (387, 493));
    gen_map.insert(5, (494, 649));
    gen_map.insert(6, (650, 721));
    gen_map.insert(7, (722, 809));
    gen_map.insert(8, (810, 898));
    gen_map
}

fn list_pokemon_names(pokemon_data: &[Pokemon]) {
    for pokemon in pokemon_data {
        println!("{}", pokemon.name);
    }
}

fn print_file_content(content: &str) {
    print!("{}", content);
}

fn show_pokemon_by_name(
    name_arg: &str,
    show_title: bool,
    is_shiny: bool,
    is_large: bool,
    form_arg: Option<&str>,
    pokemon_data: &[Pokemon],
) -> Result<()> {
    let mut name = name_arg.to_string();
    let pokemon_map: HashMap<_, _> = pokemon_data.iter().map(|p| (p.name.as_str(), p)).collect();

    if !pokemon_map.contains_key(name.as_str()) {
        return Err(anyhow!("Invalid pokemon {}", name));
    }

    if let Some(form) = form_arg {
        if let Some(pokemon) = pokemon_map.get(name.as_str()) {
            let alternate_forms: Vec<_> = pokemon.forms.iter().filter(|f| *f != "regular").collect();
            if alternate_forms.contains(&&form.to_string()) {
                name = format!("{}-{}", name, form);
            } else {
                eprintln!("Invalid form '{}' for pokemon {}", form, name);
                if alternate_forms.is_empty() {
                    eprintln!("No alternate forms available for {}", name);
                } else {
                    eprintln!("Available alternate forms are:");
                    for alt_form in alternate_forms {
                        eprintln!("- {}", alt_form);
                    }
                }
                std::process::exit(1);
            }
        }
    }

    let color_subdir = if is_shiny { "shiny" } else { "regular" };
    let size_subdir = if is_large { "large" } else { "small" };

    let pokemon_path = Path::new(size_subdir).join(color_subdir).join(&name);

    let pokemon_file = COLORSCRIPTS_DIR
        .get_file(&pokemon_path)
        .ok_or_else(|| {
            anyhow!(
                "Colorscript for '{}' not found at {}",
                name,
                pokemon_path.display()
            )
        })?;

    let content = pokemon_file.contents_utf8().ok_or_else(|| {
        anyhow!("Could not read embedded file content for '{}'", name)
    })?;

    if show_title {
        if is_shiny {
            println!("{} (shiny)", name);
        } else {
            println!("{}", name);
        }
    }

    print_file_content(content);
    Ok(())
}

fn show_random_pokemon(
    generations_str: &str,
    show_title: bool,
    is_shiny: bool,
    is_large: bool,
    pokemon_data: &[Pokemon],
) -> Result<()> {
    let gen_map = generations();
    let mut rng = rand::thread_rng();

    let (start_gen, end_gen) = if generations_str.contains(',') {
        let gens: Vec<u8> = generations_str
            .split(',')
            .filter_map(|s| s.parse().ok())
            .collect();
        if let Some(gen) = gens.choose(&mut rng) {
            (*gen, *gen)
        } else {
            return Err(anyhow!("Invalid generation list format"));
        }
    } else if generations_str.contains('-') {
        let mut parts = generations_str.split('-');
        let start = parts.next().and_then(|s| s.parse().ok());
        let end = parts.next().and_then(|s| s.parse().ok());
        if let (Some(s), Some(e)) = (start, end) {
            (s, e)
        } else {
            return Err(anyhow!("Invalid generation range format"));
        }
    } else if let Ok(gen) = generations_str.parse() {
        (gen, gen)
    } else {
        return Err(anyhow!("Invalid generation format '{}'", generations_str));
    };

    let start_id = gen_map.get(&start_gen).map_or(0, |r| r.0);
    let end_id = gen_map.get(&end_gen).map_or(0, |r| r.1);

    if start_id == 0 || end_id == 0 {
        return Err(anyhow!("Invalid generation number provided."));
    }

    let random_idx = rng.gen_range(start_id..=end_id) as usize;
    let random_pokemon_name = &pokemon_data
        .get(random_idx - 1)
        .ok_or_else(|| anyhow!("Pokemon index out of bounds"))?
        .name;
    
    let shiny = is_shiny || rng.gen_bool(SHINY_RATE);

    show_pokemon_by_name(random_pokemon_name, show_title, shiny, is_large, None, pokemon_data)
}

fn show_random_pokemon_by_names(
    names_str: &str,
    show_title: bool,
    is_shiny: bool,
    is_large: bool,
    pokemon_data: &[Pokemon],
) -> Result<()> {
    let pokemon_names: Vec<_> = pokemon_data.iter().map(|p| p.name.as_str()).collect();
    let provided_names: Vec<_> = names_str.split(',').collect();
    
    let mut valid_names = Vec::new();
    for name in provided_names {
        if pokemon_names.contains(&name) {
            valid_names.push(name);
        } else {
            eprintln!("{}", format!("Invalid pokemon {}", name).red());
        }
    }

    if valid_names.is_empty() {
        eprintln!("No correct pokemon names have been provided.");
        std::process::exit(1);
    }
    
    let mut rng = rand::thread_rng();
    let random_pokemon_name = valid_names.choose(&mut rng).unwrap();
    let shiny = is_shiny || rng.gen_bool(SHINY_RATE);

    show_pokemon_by_name(random_pokemon_name, show_title, shiny, is_large, None, pokemon_data)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let pokemon_data = load_pokemon_data()?;

    if cli.list {
        list_pokemon_names(&pokemon_data);
    } else if let Some(name) = cli.name.as_deref() {
        show_pokemon_by_name(name, cli.show_title, cli.shiny, cli.large, cli.form.as_deref(), &pokemon_data)?;
    } else if let Some(gen_str) = cli.random.as_deref() {
        if cli.form.is_some() {
            return Err(anyhow!("--form flag unexpected with --random"));
        }
        show_random_pokemon(gen_str, cli.show_title, cli.shiny, cli.large, &pokemon_data)?;
    } else if let Some(names_str) = cli.random_by_names.as_deref() {
         if cli.form.is_some() {
            return Err(anyhow!("--form flag unexpected with --random-by-names"));
        }
        show_random_pokemon_by_names(names_str, cli.show_title, cli.shiny, cli.large, &pokemon_data)?;
    } else {
        Cli::command().print_help()?;
    }

    Ok(())
} 