/**
 * Autore: Manuel Millefiori
 * Data: 2023-11-08
 * 
 * TODO:
 * Comprendere l'archiviazione tar aggiungendo
 * i file alla tarball di file compressi in gz
 * Concatenazioni stringhe delle varie path
 */

use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::thread;
use std::time::Duration;
use tar::Builder;
use flate2::Compression;
use flate2::write::GzEncoder;

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
 * @param delay
 * Delay in minuti per effettuare il backup
 * 
 * @param local_file
 * File locale da inviare al server SFTP
 * 
 * @param remote_dir
 * Cartella remota dove effettuare il backup
 * 
 * @param username
 * Username dell'utente SSH
 * 
 * @param password
 * Password dell'utente SSH
 *
 * @param remote_dir
 */
fn backup_loop(delay: u32, host: String, mut local_file: String, remote_dir: String, username: String, password: String) {
   
   // Ottengo la path del file remoto concatenando la cartella
   // di destinazione con il file locale
   local_file = get_last_segment(&local_file);

   loop {

      // Creo un backup tramite un archivio tar.gz
      let mut compressed_file = compress_tar_gz(&local_file);

      // Invio il backup compresso al server sftp
      send_file_sftp(host.clone(), local_file.clone(), (remote_dir + local_file.as_str()), username.clone(), password.clone());

      // Attendo il delay per il prossimo backup
      thread::sleep(Duration::from_secs((delay * 60) as u64));
   }
}

/**
 * @brief
 * Funzione per ottenere l'ultimo segmento della stringa
 * separandola con "/"
 */
fn get_last_segment(input: &str) -> String {

   // Dividi la stringa in segmenti utilizzando il carattere '/'
   let segments: Vec<&str> = input.split('/').collect();

   // Restituisci l'ultimo segmento se ci sono segmenti, altrimenti restituisci l'intera stringa
   return segments.last().map_or_else(|| input.to_string(), |s| s.to_string());
}

/**
 * @brief
 * Funzione che comprime un file o una cartella
 * in un archivio tar.gz
 * 
 * @param local_file
 * Nome del file da comprimere
 * 
 * @return
 * Nome dell'archivio compresso
 */
fn compress_tar_gz(local_file: &str) -> String {
   
   // Nome dell'archivio compresso da restituire
   let compressed_file = format!("{}.tar.gz", local_file);

   // Apre il file in scrittura
   let tar_gz_file = File::create(&compressed_file).expect("Impossibile creare l'archivio tar.gz");

   // Wrapper per il compressore Gzip
   let gz_encoder = GzEncoder::new(tar_gz_file, Compression::default());

   // Wrapper per il builder dell'archivio tar
   let mut tar_builder = Builder::new(gz_encoder);

   // Aggiunge il file o la cartella all'archivio
   add_file_or_folder(local_file, &mut tar_builder, "").expect("Errore durante la compressione");

   // Chiude il builder, il GzEncoder e il file compresso
   tar_builder.finish().expect("Impossibile finalizzare l'archivio tar");
   
   // Restituisco il nome del file compresso
   return compressed_file;
}

/**
 * @brief
 * Funzione ricorsiva per aggiungere file o cartelle all'archivio tar
 *
 * @param file_or_folder
 * Il percorso del file o della cartella da aggiungere all'archivio.
 * Può essere di qualsiasi tipo che implementa il trait AsRef<Path>.
 *
 * @param tar_builder
 * Il builder dell'archivio tar su cui aggiungere file o cartelle.
 *
 * @param archive_path
 * Il percorso all'interno dell'archivio tar in cui aggiungere il file o la cartella.
 * Viene utilizzato durante la ricorsione per mantenere la struttura dell'archivio.
 *
 * @return
 * Restituisce 0 se l'operazione ha avuto successo, 1 se è fallita.
 */
fn add_file_or_folder<P: AsRef<Path>>(file_or_folder: P, tar_builder: &mut Builder<GzEncoder<File>>, archive_path: &str) -> i32 {

   // Codice d'errore da restituire
   // Inizializzazione ottimistca
   let mut error = 0;

   // Converte il percorso in un tipo Path
   let file_or_folder = file_or_folder.as_ref();

   // Costruisce il percorso all'interno dell'archivio tar
   let archive_path = Path::new(archive_path).join(file_or_folder.file_name().unwrap());

   // Verifica se è un file
   if file_or_folder.is_file() {

      // Se è un file, aggiungilo all'archivio tar
      if let Err(_) = tar_builder.append_file(archive_path, file_or_folder) {

         // Aggiorno il codice d'errore
         error = 1;
      }
   } else if file_or_folder.is_dir() {

      // Se è una cartella, ricorsivamente aggiungi i suoi contenuti all'archivio tar
      for entry in file_or_folder.read_dir().unwrap() {

         let entry = entry.unwrap();

         // Se c'è un errore durante la ricorsione, restituisci 1
         if add_file_or_folder(entry.path(), tar_builder, &archive_path.to_string_lossy()) == 1 {

            // Aggiorno il codice d'errore
            error = 1;
         }
      }
   }

   // Restituisce il codice d'errore
   return error;
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
 * 5 = Errore durante la scrittura del file remoto
 * 6 = Errore durante la lettura del file locale
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
   let mut local_file = File::open(local_file_path).expect("Impossibile aprire il file locale");

   // Backup dei dati sul file remoto
   let result = sftp.create(&Path::new(remote_file_path.as_str()));

   // Gestione errori creazione file
   match result {
      
      Ok(mut remote_file) => {

         // Dimensione del buffer
         // Spedisco 1Mb alla volta
         let mut buffer = [0u8; 1048576];

            loop {

               match local_file.read(&mut buffer) {
                  
                  Ok(0) => {

                     // Fine del file
                     break;
                  }
                  Ok(bytes_read) => {

                     // Scrivo solo i bytes letti effettivamente
                     if let Err(err) = remote_file.write_all(&buffer[..bytes_read]) {

                           // DEBUG
                           eprintln!("Errore durante la scrittura sul file remoto: {:?}", err);

                           error = 5; // Imposta un codice di errore appropriato

                           break;
                     }
                  }
                  Err(err) => {

                     // DEBUG
                     eprintln!("Errore durante la lettura dal file locale: {:?}", err);

                     error = 6; // Imposta un codice di errore appropriato

                     break;
                  }
               }
            }

         // DEBUG
         println!("File inviato con successo!");

         // Chiusura delle connessioni
         drop(remote_file);
      }
      Err(err) => {

         // DEBUG
         eprintln!("{err}");
      }
   }

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
