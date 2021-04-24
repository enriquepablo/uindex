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

use std::{cell::RefCell, collections::{ HashMap }, mem};

use crate::segment::MPSegment;


pub struct Lexicon {
    segments: RefCell<HashMap<u64, Box<MPSegment>>>,
}

impl Lexicon {
    pub fn new() -> Self {
        Lexicon { 
            segments: RefCell::new(HashMap::new()),
        }
    }
    pub fn intern_with_name(&self, name: u64, text: &str, key: u64, is_leaf: bool, is_var: bool, in_var_range: bool, unique: bool) -> &MPSegment {

        let mut map = self.segments.borrow_mut();

        if !map.contains_key(&key) {
            let segment = MPSegment::new(name,
                                         text.to_string(),
                                         is_leaf,
                                         is_var,
                                         in_var_range,
                                         unique);
            map.insert(key, Box::new(segment));
        }

        let interned = map.get(&key).unwrap();

        unsafe { mem::transmute(interned.as_ref()) }
    }
}
