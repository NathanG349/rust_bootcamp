use clap::Parser;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write}; // Correction 1: Retrait de `self`
use std::process;

/// Outil de lecture et écriture binaire en hexadécimal
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Fichier cible
    #[arg(short, long)]
    file: String,

    /// Mode lecture (affiche le hex dump)
    #[arg(short, long)]
    read: bool,

    /// Mode écriture (chaine hexadécimale à écrire)
    #[arg(short, long)]
    write: Option<String>,

    /// Offset en bytes (décimal ou hex avec 0x)
    #[arg(short, long, default_value = "0")]
    offset: String,

    /// Nombre de bytes à lire (pour le mode lecture)
    #[arg(short, long, default_value_t = 16)]
    size: usize,
}

fn main() {
    let args = Args::parse();

    let offset = parse_offset(&args.offset).unwrap_or_else(|err| {
        eprintln!("Erreur offset invalide: {}", err);
        process::exit(1);
    });

    if let Some(hex_data) = args.write {
        handle_write(&args.file, &hex_data, offset);
    } else if args.read {
        handle_read(&args.file, offset, args.size);
    } else {
        use clap::CommandFactory;
        Args::command().print_help().unwrap();
    }
}

fn handle_write(path: &str, hex_str: &str, offset: u64) {
    let bytes = match decode_hex(hex_str) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Erreur de format hexadécimal: {}", e);
            return;
        }
    };

    println!("Writing {} bytes at offset {:#010x}", bytes.len(), offset);

    // Correction 2: Ajout de .truncate(false)
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false) 
        .open(path)
        .expect("Impossible d'ouvrir le fichier");

    file.seek(SeekFrom::Start(offset))
        .expect("Erreur lors du seek");

    file.write_all(&bytes).expect("Erreur lors de l'écriture");

    print_bytes_info(&bytes);
    println!("✓ Successfully written");
}

fn handle_read(path: &str, offset: u64, size: usize) {
    let mut file = File::open(path).expect("Fichier introuvable");

    file.seek(SeekFrom::Start(offset))
        .expect("Erreur lors du seek");

    let mut buffer = vec![0; size];
    let bytes_read = file.read(&mut buffer).expect("Erreur de lecture");

    buffer.truncate(bytes_read);

    for (i, chunk) in buffer.chunks(16).enumerate() {
        let current_offset = offset + (i * 16) as u64;

        print!("{:08x}: ", current_offset);

        for byte in chunk {
            print!("{:02x} ", byte);
        }

        for _ in 0..(16 - chunk.len()) {
            print!("   ");
        }

        print!("|");
        for byte in chunk {
            if byte.is_ascii_graphic() || *byte == 0x20 {
                print!("{}", *byte as char);
            } else {
                print!(".");
            }
        }
        println!("|");
    }
}

fn parse_offset(input: &str) -> Result<u64, String> {
    let input = input.trim();
    // Correction 3: Utilisation de strip_prefix
    if let Some(stripped) = input.strip_prefix("0x") {
        u64::from_str_radix(stripped, 16).map_err(|_| "Hex invalide".to_string())
    } else {
        input
            .parse::<u64>()
            .map_err(|_| "Nombre invalide".to_string())
    }
}

fn decode_hex(s: &str) -> Result<Vec<u8>, String> {
    // Correction 4: Utilisation de is_multiple_of
    if !s.len().is_multiple_of(2) {
        return Err("La longueur de la chaîne hex doit être paire".to_string());
    }

    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|_| format!("Caractère invalide à l'index {}", i))
        })
        .collect()
}

fn print_bytes_info(bytes: &[u8]) {
    print!("Hex: ");
    for b in bytes {
        print!("{:02x} ", b);
    }
    println!();

    print!("ASCII: ");
    for b in bytes {
        if b.is_ascii_graphic() || *b == 0x20 {
            print!("{}", *b as char);
        } else {
            print!(".");
        }
    }
    println!();
}
