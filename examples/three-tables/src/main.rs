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

    /// take batch of samples every so many facts
    #[structopt(short, long)]
    report: usize,
}

fn main() {
    let opt = Opt::from_args();
    env_logger::init();
    let db = kb::IsaGen::gen_db();

    //let o_one_sec = time::Duration::from_millis(100);
    let t0 = SystemTime::now();
    let mut start = 0;
    let mut count = 0;
    
    for i in 0..opt.facts {
        start += 1;
        
        let userid = format!("user{}", start);
        let given_name = format!("John{}", start);
        let surname = format!("Smith{}", start);

        let pre_f1 = format!("U {} {} {} ◊", given_name, surname, userid);
        let f1 = Box::leak(Box::new(pre_f1));

        let street = format!("Lane{}", start % 1000);
        let number = format!("{}", start);
        let city = format!("city{}", start % 100);

        let pre_f2 = format!("A {} {} {} {} ◊", userid, street, number, city);
        let f2 = Box::leak(Box::new(pre_f2));

        let t1 = SystemTime::now();

        db.tell( unsafe { mem::transmute( f1.as_str() ) });
        db.tell( unsafe { mem::transmute( f2.as_str() ) });
        count += 2;

        if i < 100 {
            let population = format!("{}", (start * 1000));
            let country = format!("country{}", start % 50);

            let pre_f3 = format!("T {} {} {} ◊", city, population, country);
            let f3 = Box::leak(Box::new(pre_f3));
            db.tell( unsafe { mem::transmute( f3.as_str() ) });
            count += 1;
        }
        
        let t2 = SystemTime::now();

        if (i % opt.report) == 0 {

            let t_f = t2.duration_since(t1).unwrap().as_micros() as f64;

            let f = Box::leak(Box::new(format!("U {} {} X1 ◊ A X1 X2 X3 X4 ◊ T X4 X5 X6 ◊", given_name, surname)));

            let resp = db.ask( unsafe { mem::transmute( f.as_str() ) });

            if resp.len() != 1 {
                println!("Wrong resp for {}: found {:?}", f, resp);
            }
            let t3 = SystemTime::now();

            let t_q = t3.duration_since(t2).unwrap().as_micros() as f64;

            println!("  round {}, duration: fact {} usec, query {} usec", i, t_f, t_q);
        }
    }
    let t3 = SystemTime::now();
    let total_time = t3.duration_since(t0).unwrap().as_millis() as f64 / 1000.0;

    println!("total time: {} sec for {} entries", total_time, count);

}
