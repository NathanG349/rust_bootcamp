use clap::Parser;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
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

    // 1. Parsing de l'offset (qui peut être "10" ou "0xA")
    let offset = parse_offset(&args.offset).unwrap_or_else(|err| {
        eprintln!("Erreur offset invalide: {}", err);
        process::exit(1);
    });

    if let Some(hex_data) = args.write {
        // --- MODE ÉCRITURE ---
        handle_write(&args.file, &hex_data, offset);
    } else if args.read {
        // --- MODE LECTURE ---
        handle_read(&args.file, offset, args.size);
    } else {
        // Si aucun mode n'est choisi, on affiche l'aide
        use clap::CommandFactory;
        Args::command().print_help().unwrap();
    }
}

// Fonction pour écrire dans le fichier
fn handle_write(path: &str, hex_str: &str, offset: u64) {
    // Conversion de la string hex ("4865") en vecteur de bytes ([0x48, 0x65])
    let bytes = match decode_hex(hex_str) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Erreur de format hexadécimal: {}", e);
            return;
        }
    };

    println!("Writing {} bytes at offset {:#010x}", bytes.len(), offset);

    // Ouverture du fichier en mode écriture (create si inexistant, write pour modifier)
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
        .expect("Impossible d'ouvrir le fichier");

    // On déplace le curseur à l'offset voulu
    file.seek(SeekFrom::Start(offset)).expect("Erreur lors du seek");
    
    // On écrit les données
    file.write_all(&bytes).expect("Erreur lors de l'écriture");

    // Feedback utilisateur (comme sur l'image)
    print_bytes_info(&bytes);
    println!("✓ Successfully written");
}

// Fonction pour lire le fichier
fn handle_read(path: &str, offset: u64, size: usize) {
    let mut file = File::open(path).expect("Fichier introuvable");

    // On se déplace
    file.seek(SeekFrom::Start(offset)).expect("Erreur lors du seek");

    // On prépare un buffer de la taille demandée
    let mut buffer = vec![0; size];
    let bytes_read = file.read(&mut buffer).expect("Erreur de lecture");

    // On redimensionne le buffer si on a lu moins que prévu (fin de fichier)
    buffer.truncate(bytes_read);

    // Affichage formaté façon "Hex Dump"
    // On itère par blocs de 16 (standard hex dump)
    for (i, chunk) in buffer.chunks(16).enumerate() {
        let current_offset = offset + (i * 16) as u64;
        
        // 1. Affichage de l'offset
        print!("{:08x}: ", current_offset);

        // 2. Affichage des bytes en hex
        for byte in chunk {
            print!("{:02x} ", byte);
        }

        // Padding (alignement) si la ligne fait moins de 16 bytes
        for _ in 0..(16 - chunk.len()) {
            print!("   ");
        }

        // 3. Affichage ASCII
        print!("|");
        for byte in chunk {
            if byte.is_ascii_graphic() || *byte == 0x20 { // 0x20 est l'espace
                print!("{}", *byte as char);
            } else {
                print!(".");
            }
        }
        println!("|");
    }
}

// Helper pour parser l'offset (gère "0x" ou décimal)
fn parse_offset(input: &str) -> Result<u64, String> {
    let input = input.trim();
    if input.starts_with("0x") {
        u64::from_str_radix(&input[2..], 16).map_err(|_| "Hex invalide".to_string())
    } else {
        input.parse::<u64>().map_err(|_| "Nombre invalide".to_string())
    }
}

// Helper pour convertir "48656c" -> vec![0x48, 0x65, 0x6c]
fn decode_hex(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
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

// Helper pour l'affichage Hex/ASCII du feedback d'écriture
fn print_bytes_info(bytes: &[u8]) {
    print!("Hex: ");
    for b in bytes { print!("{:02x} ", b); }
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