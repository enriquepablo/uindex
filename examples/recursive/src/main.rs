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

use rand;

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

    /// depth of trees
    #[structopt(short, long)]
    treedepth: usize,

    /// length of branches
    #[structopt(short, long)]
    branchlength: usize,
}

pub fn random (top: usize) -> usize {
    (rand::random::<f64>() * top as f64) as usize
}

fn make_tree(depth: usize, length: usize) -> String {
    let mut tree = do_make_tree(depth, depth, length);
    tree.push_str(" <>");
    tree
}

fn do_make_tree(depth: usize, max_depth: usize, length: usize) -> String {
    if depth == 0 || (depth != max_depth && random(8) > 5) {
        return format!("{}", random(usize::pow(10, (max_depth - depth + 1) as u32)));
    }
    let mut tree = String::new();
    tree.push('(');
    tree.push_str(&format!("{}", random(usize::pow(10, (max_depth - depth + 1) as u32))));
    for _ in 0..random(length) {
        tree.push(' ');
        tree.push_str(&do_make_tree(depth - 1, max_depth, length));
    }
    tree.push(')');
    tree
}

fn make_tree_full(depth: usize, length: usize) -> String {
    let mut tree = do_make_tree_full(depth, depth, length);
    tree.push_str(" <>");
    tree
}


fn do_make_tree_full(depth: usize, max_depth: usize, length: usize) -> String {
    if depth == 0 {
        return format!("{}", random(usize::pow(10, (max_depth - depth + 1) as u32)));
    }
    let mut tree = String::new();
    tree.push('(');
    tree.push_str(&format!("{}", random(usize::pow(10, (max_depth - depth + 1) as u32))));
    for _ in 0..length {
        tree.push(' ');
        tree.push_str(&do_make_tree_full(depth - 1, max_depth, length));
    }
    tree.push(')');
    tree
}

fn main() {
    let opt = Opt::from_args();
    env_logger::init();
    let db = kb::IsaGen::gen_db();

    //let o_one_sec = time::Duration::from_millis(100);
    let t0 = SystemTime::now();
    let mut count = 0;
    
    for i in 0..opt.facts {
        let f = Box::leak(Box::new(make_tree(opt.treedepth, opt.branchlength)));
        db.tell( unsafe { mem::transmute( f.as_str() ) });
        count += 1;

        if (i % opt.report) == 0 {
            let q = Box::leak(Box::new(make_tree_full(opt.treedepth, opt.branchlength)));
            let t1 = SystemTime::now();
            db.tell( unsafe { mem::transmute( q.as_str() ) });
            count += 1;
            let t2 = SystemTime::now();

            let t_f = t2.duration_since(t1).unwrap().as_micros() as f64;

            let resp = db.ask( unsafe { mem::transmute( q.as_str() ) });
            if resp.len() != 1 {
                println!("Wrong resp for {}: found {}, expected {}", f, resp.len(), 1);
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
