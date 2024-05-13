use min_max::*;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::io::{self, Write, BufReader, BufRead};
use std::sync::mpsc;
use std::thread;

pub mod gotoh;
pub mod cmpmatrix;

#[derive(Clone,Copy)]
struct WorkResult {
    row: usize,
    col: usize,
    val: f32,
}

fn main() -> std::io::Result<()> {
    //let a = String::from("abc");
    //let b = String::from("abc");
    //let a = String::from("Hello, world!");
    //let b = String::from("Hellp, world!");
    //let b = String::from("Hsddsfasdfasd:wellp, world!");

    let mut mtx = cmpmatrix::CmpMatrix::new();
    let infile = File::open("outputs.txt")?;
    let reader = BufReader::new(infile);

    for ln in reader.lines() {
        match ln {
            Ok(s) => {
                let splits: Vec<&str> = s.split(":").collect();
                mtx.add(splits[0].to_string(), splits[1].to_string(), splits[2].to_string());
            },
            Err(_) => {
            }
        }
    };

    let total = mtx.entries_len()*mtx.entries_len();

    let mut output_file = File::create("results.csv")?;

    let entry_count = mtx.entries_len();

    write!(output_file, "algo\tsig_name\tsig_len\t").unwrap();

    let mut work: Vec<Vec<usize>> = vec![];

    for i in 0..entry_count {
        match mtx.get_entry(i) {
            Ok(e) => {
                write!(output_file, "{}-{}\t", e.module, e.symbol).unwrap();
                for j in 0..entry_count {
                    work.push(vec![i, j]);
                };
            },
            Err(_) => {
            },
        };
    }

    write!(output_file, "\n").unwrap();

    let (tx, rx) = mpsc::channel();
    let mut thread_list = vec![];
    let work_queue = Arc::new(Mutex::new(work));
    const NTHREADS: u16 = 24;

    for _tindex in 0..NTHREADS {
        // Gets a clone()d vec of the entries
        let entries = mtx.get_entries();

        let tx_clone = tx.clone();
        let work_queue_clone = Arc::clone(&work_queue);
        thread_list.push(
            thread::spawn(move || {
                let mut gotoh_compare = gotoh::GotohInstance::new(1000, 100, 1000);
                loop {
                    let mut wq = work_queue_clone.lock().unwrap();
                    if (*wq).len() == 0 {
                        drop(wq); // This will unlock the Mutex after we've localized the work data
                        break;
                    }

                    let work_item = (*wq).pop().unwrap();

                    drop(wq); // This will unlock the Mutex after we've localized the work data

                    let v = /*(maxlen * 1000.) as isize - */gotoh_compare.init(&entries[work_item[0]].sig, &entries[work_item[1]].sig);
                    let minlen = min!(entries[work_item[0]].sig.len(), entries[work_item[1]].sig.len()) as f32;
                    let res = ((v as f32) + 1000.*minlen) / (1000.*minlen*2.);

                    tx_clone.send(WorkResult {row: work_item[1], col: work_item[0], val: res}).unwrap();
                };
            })
        );
    };

    // Drop the original tx so it doesn't hold the receiver open
    drop(tx);

    let mut recvcount = 0;
    let wq = Arc::clone(&work_queue);
    for recvmsg in rx {
        mtx.update_by_index(recvmsg.col, recvmsg.row, recvmsg.val).unwrap();
        recvcount = recvcount + 1;
        if recvcount % 100 == 0 {
            let v = wq.lock().unwrap();
            let completed = v.len();
            drop(v);
            print!("{} / {} ({})                     \r", recvcount, total, completed);
            io::stdout().flush().unwrap();
        }
    }

    print!("\n");

    // Do thread cleanup here
    for th in thread_list {
        th.join().unwrap();
    };

    // Write output to CSV
    for j in 0..mtx.entries_len() {
        let this_entry = mtx.get_entries()[j].clone();
        write!(output_file, "Gotoh\t{}-{}\t{}\t", this_entry.module, this_entry.symbol, this_entry.sig.len()).unwrap();
        for i in 0..mtx.entries_len() {
            write!(output_file, "{}\t", mtx.get_compare_val(i, j)).unwrap();
        }
        write!(output_file, "\n").unwrap();
    }

    Ok(())
}
