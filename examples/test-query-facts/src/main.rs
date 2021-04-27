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


use crate::uindex::kbase::DBGen;
use crate::uindex::kbase::DataBase;

mod kb;


fn main() {
    env_logger::init();
    let db = kb::IsaGen::gen_db();

    let f1 = "john ISA0 person ◊";
    db.tell( f1 );

    let f2 = "john ISA0 animal ◊";
    db.tell( f2 );

    let f3 = "sue ISA0 animal ◊";
    db.tell( f3 );

    let f4 = "mary ISA0 animal ◊";
    db.tell( f4 );

    let f5 = "john ISA0 philosopher ◊";
    db.tell( f5 );

    let f6 = "sue ISA0 person ◊";
    db.tell( f6 );

    let q1 = "john ISA0 X1 ◊";
    let resp1 = db.ask( q1 );

    println!("John ISA\n\n{:#?}", resp1);

    let q2 = "sue ISA0 X1 ◊ john ISA0 X1 ◊";
    let resp2 = db.ask( q2 );

    println!("John and SUE ISA\n\n{:#?}", resp2);

    let q3 = "sue ISA0 X1 ◊ john ISA0 X1 ◊ mary ISA0 X1 ◊";
    let resp3 = db.ask( q3 );

    println!("John and SUE and mary ISA\n\n{:#?}", resp3);
}
