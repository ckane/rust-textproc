use min_max::*;
use std::fs::File;
use std::sync::Arc;
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

    write!(output_file, "algo\tsig_name\tsig_len\t");

    for i in 0..entry_count {
        match mtx.get_entry(i) {
            Ok(e) => {
                write!(output_file, "{}-{}\t", e.module, e.symbol);
            },
            Err(_) => {
            },
        };
    }

    write!(output_file, "\n");

    let (tx, rx) = mpsc::channel();
    let mut thread_list = vec![];
    const nthreads: u16 = 48;

    for tindex in 0..nthreads {
        // Gets a clone()d vec of the entries
        let entries = mtx.get_entries();

        let tx_clone = tx.clone();
        thread_list.push(
            thread::spawn(move || {
                let mut gotoh_compare = gotoh::GotohInstance::new(10, 4, 10);
                let step_size = entries.len() / nthreads as usize;
                let start_pos = step_size * tindex as usize;
                let this_set = if tindex == nthreads - 1 {
                    match entries.len() % step_size {
                        0 => {
                            step_size
                        },
                        _ => {
                            entries.len() % step_size
                        }
                    }
                } else {
                    step_size
                };

                for j in start_pos..(start_pos + this_set) {
                    for i in 0..entries.len() {
                        let v = gotoh_compare.init(&entries[i].sig, &entries[j].sig);
                        let maxlen = max!(entries[i].sig.len(), entries[j].sig.len()) as f32;
                        let minv = -maxlen * 1.;
                        let res = ((v as f32) - 10.*minv) / (10.*maxlen*2.);

                        tx_clone.send(WorkResult {row: j, col: i, val: res}).unwrap();
                    };
                };
            })
        );
    }

    let mut recvcount = 0;
    for recvmsg in rx {
        mtx.update_by_index(recvmsg.col, recvmsg.row, recvmsg.val).unwrap();
        recvcount = recvcount + 1;
        if recvcount % 100 == 0 {
            print!("{} / {}\r", recvcount, total);
            io::stdout().flush().unwrap();
        }
    }

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