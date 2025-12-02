use clap::Parser;
use std::collections::HashMap;
use std::io::{self, Read};

/// Compteur de fréquence de mots dans un texte
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    text: Option<String>,

    #[arg(short, long, default_value_t = 10)]
    top: usize,

    #[arg(short, long, default_value_t = 1)]
    min_length: usize,

    /// Comptage
    #[arg(short, long)]
    ignore_case: bool,
}

fn main() {
    let args = Args::parse();

    // 1. Récupération du contenu
    let content = match args.text {
        Some(text) => text,
        None => {
            let mut buffer = String::new();

            io::stdin()
                .read_to_string(&mut buffer)
                .expect("Erreur de lecture stdin");
            buffer
        }
    };

    // 2. Comptage des mots
    let mut word_counts: HashMap<String, u32> = HashMap::new();

    for raw_word in content.split_whitespace() {
        // Nettoyage
        let mut word = raw_word
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_string();

        if args.ignore_case {
            word = word.to_lowercase();
        }

        if word.len() >= args.min_length {
            *word_counts.entry(word).or_insert(0) += 1;
        }
    }

    // 3. Tri des résultats

    let mut sorted_words: Vec<(&String, &u32)> = word_counts.iter().collect();

    sorted_words.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

    // 4. Affichage

    if !sorted_words.is_empty() {
        println!("Word frequency:");
        for (word, count) in sorted_words.into_iter().take(args.top) {
            println!("{}: {}", word, count);
        }
    }
}
