use std::{io, time::Instant};

use coil::{Index, tokenize, hash_str};



fn main() {
    let mut index = Index::new(1);

    let mut path = String::new();
    print!("Enter path of file to index");
    io::stdin().read_line(&mut path).expect("failed to read line");
    let path = path.trim();

    let start = Instant::now();
    let mut num_docs = 0;


    let mut rdr = csv::Reader::from_path(path).expect("could not open file");
    for (_, result) in rdr.records().enumerate() {
        let record = result.expect("invalid csv record");
        let title = record.get(1).expect("missing job title");
        let description = record.get(2).expect("missing job title");
        let tokens = tokenize(description);

        for (tok, term_freq) in tokens.into_iter() {
            let tok_id = hash_str(&tok);
            let mut score = vec![term_freq as i32];
            index.index(tok_id, num_docs, &mut score, title.to_string()).unwrap();
        }
        num_docs += 1;
    }
    let end = Instant::now();
    println!("Indexing took: {}", (end - start).as_secs_f64());
    println!("Num docs indexed: {}", index.num_docs());

    loop  {
        let mut query = String::new();
        io::stdin().read_line(&mut query).expect("failed to read line");
        let query = query.trim();
        if query.eq("quit") {
            break;
        }
        let start = Instant::now();
        let results = index.search(query, 10);
        let end = Instant::now();
        println!("took {} seconds", (end - start).as_secs_f64());
        for (result, score) in results {
            println!("{result}: {score:?}");
        }
        
    }


}
