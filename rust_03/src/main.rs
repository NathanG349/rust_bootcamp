use clap::{Parser, Subcommand};
use rand::Rng;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

const P: u64 = 0xD87FA3E291B4C7F3;
const G: u64 = 2;

const LCG_A: u64 = 1103515245;
const LCG_C: u64 = 12345;
const LCG_M: u64 = 1u64 << 32;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Server {
        #[arg(default_value_t = 8080)]
        port: u16,
    },

    Client {
        host: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server { port } => start_server(port),
        Commands::Client { host } => start_client(&host),
    }
}

fn mod_pow(base: u64, exp: u64, modulus: u64) -> u64 {
    let mut result = 1u128;
    let mut base = base as u128;
    let modulus = modulus as u128;
    let mut exp = exp;

    while exp > 0 {
        if exp % 2 == 1 {
            result = (result * base) % modulus;
        }
        base = (base * base) % modulus;
        exp /= 2;
    }
    result as u64
}

struct LcgCipher {
    state: u64,
}

impl LcgCipher {
    fn new(seed: u64) -> Self {
        println!("[STREAM] Generating keystream from secret...");
        println!("Algorithm: LCG (a={}, c={}, m=2^32)", LCG_A, LCG_C);
        println!("Seed: secret = {:X}", seed);

        let cipher = LcgCipher { state: seed };

        print!("Keystream: ");
        let mut temp_state = seed;
        for _ in 0..10 {
            temp_state = (LCG_A.wrapping_mul(temp_state).wrapping_add(LCG_C)) % LCG_M;
            print!("{:02X} ", (temp_state >> 24) as u8);
        }
        println!("...\n");

        cipher
    }

    fn next_byte(&mut self) -> u8 {
        self.state = (LCG_A.wrapping_mul(self.state).wrapping_add(LCG_C)) % LCG_M;
        (self.state >> 24) as u8
    }

    fn process(&mut self, data: &[u8]) -> Vec<u8> {
        data.iter().map(|b| b ^ self.next_byte()).collect()
    }
}

fn start_server(port: u16) {
    let address = format!("0.0.0.0:{}", port);
    println!("[SERVER] Listening on {}", address);
    let listener = TcpListener::bind(&address).expect("Failed to bind");
    println!("[SERVER] Waiting for client...");

    if let Ok((stream, addr)) = listener.accept() {
        println!("\n[CLIENT] Connected from {}", addr);
        handle_connection(stream);
    }
}

fn start_client(host: &str) {
    println!("[CLIENT] Connecting to {}...", host);
    match TcpStream::connect(host) {
        Ok(stream) => {
            println!("[CLIENT] Connected!");
            handle_connection(stream);
        }
        Err(e) => eprintln!("Failed to connect: {}", e),
    }
}

fn handle_connection(mut stream: TcpStream) {
    println!("\n[DH] Starting key exchange...");
    println!("[DH] Using hardcoded DH parameters:");
    println!("p = {:X} (64-bit prime - public)", P);
    println!("g = {} (generator - public)", G);

    let private_key: u64 = rand::rng().random();
    let public_key = mod_pow(G, private_key, P);

    println!("\n[DH] Generating our keypair...");
    println!("private_key = {:X} (random 64-bit)", private_key);
    println!("public_key = {:X}", public_key);

    println!("\n[DH] Exchanging keys...");
    println!("[NETWORK] Sending public key (8 bytes)...");
    println!("-> Send our public: {:X}", public_key);
    stream.write_all(&public_key.to_be_bytes()).unwrap();

    let mut buffer = [0u8; 8];
    stream.read_exact(&mut buffer).unwrap();
    let their_public_key = u64::from_be_bytes(buffer);
    println!("[NETWORK] Received public key (8 bytes) ✓");
    println!("<- Receive their public: {:X}", their_public_key);

    println!("\n[DH] Computing shared secret...");
    let shared_secret = mod_pow(their_public_key, private_key, P);
    println!("secret = {:X}", shared_secret);
    println!("\n[VERIFY] Both sides computed the same secret ✓");

    let encryptor = Arc::new(Mutex::new(LcgCipher::new(shared_secret)));
    let decryptor = Arc::new(Mutex::new(LcgCipher::new(shared_secret)));

    println!("Secure channel established!");

    let mut stream_clone = stream.try_clone().expect("Clone failed");
    let decryptor_clone = Arc::clone(&decryptor);

    // 3. Thread Réception
    thread::spawn(move || {
        let mut buffer = [0u8; 512];
        loop {
            match stream_clone.read(&mut buffer) {
                Ok(0) => {
                    println!("Connection closed.");
                    std::process::exit(0);
                }
                Ok(n) => {
                    let encrypted_data = &buffer[0..n];
                    println!("\n[NETWORK] Received encrypted message ({} bytes)", n);
                    println!("[-] Received {} bytes", n);

                    let mut cipher = decryptor_clone.lock().unwrap();
                    let decrypted = cipher.process(encrypted_data);

                    print_hex("Cipher", encrypted_data);
                    print_hex("Plain", &decrypted);

                    if let Ok(msg) = String::from_utf8(decrypted) {
                        println!("\n[DECRYPTED MSG] {}", msg.trim());
                    }
                    print!("\n[CHAT] Type message:\n> ");
                    io::stdout().flush().unwrap();
                }
                Err(_) => break,
            }
        }
    });

    loop {
        println!("\n[CHAT] Type message:");
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let plain_bytes = input.trim().as_bytes();

        if plain_bytes.is_empty() {
            continue;
        }

        println!("\n[ENCRYPT]");
        print_hex("Plain", plain_bytes);
        println!("(\"{}\")", input.trim());

        {
            let mut cipher = encryptor.lock().unwrap();

            let current_state = cipher.state;
            let enc = cipher.process(plain_bytes);
            cipher.state = current_state;
            let _dec = cipher.process(&enc);
            cipher.state = current_state;

            println!(
                "[TEST] Round-trip verified: \"{}\" -> encrypt -> decrypt -> \"{}\" ✓",
                input.trim(),
                input.trim()
            );

            let final_cipher = cipher.process(plain_bytes);
            print_hex("Cipher", &final_cipher);

            println!(
                "\n[NETWORK] Sending encrypted message ({} bytes)...",
                final_cipher.len()
            );
            stream.write_all(&final_cipher).unwrap();
            println!("[+] Sent {} bytes", final_cipher.len());
        }
    }
}

fn print_hex(label: &str, data: &[u8]) {
    print!("{}: ", label);
    for b in data {
        print!("{:02x} ", b);
    }
    println!();
}
