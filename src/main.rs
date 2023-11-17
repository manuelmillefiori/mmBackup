/**
 * Autore: Manuel Millefiori
 * Data: 2023-11-08
 */

use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::fs::File;

use ssh2::Session;

// Main
fn main() {

   // Ottengo un array di stringhe contenente
   // tutti gli argomenti passati al main
   let args: Vec<String> = env::args().collect();

   // Ciclo per stampare gli argomenti
   /*
   for (i, arg) in args.iter().enumerate() {
      println!("Argomento {}: {}", i, arg);
   }
   */

   // Verifico il corretto avvio del programma
   if correct_start(&args) {

      // Ottengo il numero di minuti di delay per effettuare un backup
      let delay: Result<u32, _> = args[1].parse();

      // Verifico se la conversione è andata a buon fine
      match delay {
         Ok(delay) => {

            // DEBUG
            println!("Backup delay set: {}", delay);

            // Avvio il loop per la gestione dei backup
            backup_loop(delay, args[2].clone(), args[3].clone(), args[4].clone(), args[5].clone(), args[6].clone());
         }
         Err(_) => {

            println!("For more info on how to use mmbackup digit:\nmmbackup --help");
         }
      }
   }
   else {
      
   }
   
}

/**
 * @brief
 * Funzione che esegue un backup di una determinata
 * cartella locale su un server remoto tramite il protocollo
 * sftp ogni delay (il delay è definito in minuti)
 * 
 * TODO:
 * Aggiornamento parametri
 * 
 * @param delay
 * Delay in minuti per effettuare il backup
 * 
 * @param local_file
 * Cartella locale da gestire
 * 
 * TODO:
 * Gestione cartella remota (address, protocol, path_dir)
 * @param remote_dir
 */
fn backup_loop(delay: u32, host: String, local_file: String, remote_dir: String, username: String, password: String) {
   
   // TODO:
   // Sostituire remote_dir gestendolo con la compressione della cartella
   send_file_sftp(host, local_file, remote_dir, username, password);
}

/**
 * @brief
 * Funzione per inviare un file locale su un server remoto
 * tramite il protocollo SFTP
 * 
 * @param host
 * Indirizzo del server SSH
 * 
 * @param local_file_path
 * PATH del file in locale da inviare
 * 
 * @param remote_file_path
 * PATH del file di destinazione
 * 
 * @param username
 * Username dell'utente SSH
 * 
 * @param password
 * Password dell'utente SSH
 * 
 * @return
 * 1 = Errore durante l'handshake
 * 2 = Errore durante l'autenticazione
 * 3 = Autenticazione fallita
 * 4 = Servizio SFTP non disponibile
 */
fn send_file_sftp(host: String, local_file_path: String, remote_file_path: String, username: String, password: String) -> u16 {

   // Errore
   // Inizializzazione ottimistica
   let mut error = 0;

   // Definisco la porta del server SSH
   let port = 22;

   let tcp = TcpStream::connect((host, port)).unwrap();

   // Connessione al server SSH
   let mut sess = Session::new().unwrap();

   sess.set_tcp_stream(tcp);

   // Handshake SSL
   sess.handshake().unwrap();

   // Autenticazione
   sess.userauth_password(username.as_str(), password.as_str()).unwrap();

   if !sess.authenticated() {
      
      return 3;
   }

   // Inizializzazione dell'oggetto SFTP sulla sessione SSH
   let sftp = sess.sftp().unwrap();

   // Apertura del file locale
   let local_file = File::open(local_file_path).expect("Impossibile aprire il file locale");

   // Backup dei dati sul file remoto
   let mut remote_file = sftp.create(&Path::new(remote_file_path.as_str())).unwrap();
   let mut buffer = Vec::new();

   // Ottengo tutti i dati del file locale
   local_file.take(u64::MAX as u64).read_to_end(&mut buffer).unwrap();

   // Scrivo tutti i dati sul file remoto
   remote_file.write_all(&buffer).unwrap();

   // DEBUG
   println!("File inviato con successo!");

   // Chiusura delle connessioni
   drop(remote_file);

   // Restituisco il codice d'errore
   return error;
}

/**
 * @brief
 * Funzione per verificare il corretto avvio del programma.
 * L'esito della funzione definisce anche la possibilità
 * di effettuare un parse del 2 argomento della chiamata
 * del programma da String a u32
 * 
 * @param args
 * Vettore di stringhe contenente tutti gli argomenti
 * passati all'avvio del programma
 * 
 * @return
 * true = Programma avviato correttamente
 * false = Sintassi errata nell'avvio del programma
 */
fn correct_start(args: &Vec<String>) -> bool {

   // Flag per verificare che il programma sia stato avviato
   // correttamente
   let mut correct_start = false;

   // Verifica corretto avvio programma
   if args.len() == 7 {

      // Aggiorno il flag
      correct_start = true;
   }
   else if args.len() > 1 && args[1] == "--help" {

      println!("mmbackup [delay: in minutes] [host] [local_file] [remote_dir] [username] [password]");
   }
   else {

      println!("For more info on how to use mmbackup digit:\nmmbackup --help");
   }

   // Restituisco il flag
   return correct_start;
}
