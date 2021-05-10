// Copyright (c) 2020 by Enrique Pérez Arnaud <enrique at cazalla.net>    
//    
// This file is part of the modus_ponens project.    
// http://www.modus_ponens.net    
//    
// The modus_ponens project is free software: you can redistribute it and/or modify    
// it under the terms of the GNU General Public License as published by    
// the Free Software Foundation, either version 3 of the License, or    
// (at your option) any later version.    
//    
// The modus_ponens project is distributed in the hope that it will be useful,    
// but WITHOUT ANY WARRANTY; without even the implied warranty of    
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the    
// GNU General Public License for more details.    
//    
// You should have received a copy of the GNU General Public License    
// along with any part of the modus_ponens project.    
// If not, see <http://www.gnu.org/licenses/>.

extern crate uindex;
#[macro_use]
extern crate uindex_derive;

extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::mem;
use std::time::SystemTime;
use structopt::StructOpt;
//use std::{thread, time};


use crate::uindex::kbase::DBGen;
use crate::uindex::kbase::DataBase;

mod kb;


#[derive(Debug, StructOpt)]
#[structopt(name = "isa-benchmark", about = "Benchmarking modus_ponens.")]
struct Opt {
    /// number of facts
    #[structopt(short, long)]
    facts: usize,

    /// take batch of samples every so many rules
    #[structopt(short, long)]
    report: usize,
}

fn main() {
    let opt = Opt::from_args();
    env_logger::init();
    let db = kb::IsaGen::gen_db();

    let sets = ["thing", "animal", "mammal", "primate", "human"];
    let nsets = 5;
        
    //let o_one_sec = time::Duration::from_millis(100);
    let t0 = SystemTime::now();
    let mut start = 0;
    
    for i in 0..opt.facts {
        start += 1;
        let s = sets[(i % nsets) as usize];
        let name = format!("{}{}{}", s, i, start);
        let f = Box::leak(Box::new(format!("{name} ISA{start} {s} ◊", name = name, start = start, s = s)));
        db.tell( unsafe { mem::transmute( f.as_str() ) });

        if (i % opt.report) == 0 {
            let t1 = SystemTime::now();
            start += 1;
            let f = Box::leak(Box::new(format!("susan ISA{start} person ◊ johnny ISA{start} person ◊", start = start)));
            db.tell( unsafe { mem::transmute( f.as_str() ) });
            let t2 = SystemTime::now();

            let t_f = t2.duration_since(t1).unwrap().as_micros() as f64 / 2.0;

            let f = Box::leak(Box::new(format!("johnny ISA{start} <X1> ◊ susan ISA{start} <X1> ◊", start = start)));
            let resp = db.ask( unsafe { mem::transmute( f.as_str() ) });
            if resp.len() == 0 {
                println!("Wrong resp for {}", f);
            }
            let t3 = SystemTime::now();

            let t_q = t3.duration_since(t2).unwrap().as_micros() as f64;

            println!("{}   {}", t_f, t_q);
        }
    }
    let t3 = SystemTime::now();
    let total_time = t3.duration_since(t0).unwrap().as_millis() as f64 / 1000.0;

    println!("total time: {} sec for {} entries", total_time, start);

}
