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

use std::{cell::RefCell, collections::{ HashMap, HashSet }, mem};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::constants;
use crate::segment::MPSegment;


pub struct Lexicon {
    segments: RefCell<HashMap<u64, Box<MPSegment>>>,
    names: RefCell<HashSet<String>>,
}

impl Lexicon {
    pub fn new() -> Self {
        Lexicon { 
            segments: RefCell::new(HashMap::new()),
            names: RefCell::new(HashSet::new()),
        }
    }
    fn calculate_hash(&self, name: &str, text: &str, is_leaf: bool) -> u64 {
        let mut s = DefaultHasher::new();
        name.hash(&mut s);
        text.hash(&mut s);
        is_leaf.hash(&mut s);
        s.finish()
    }
    pub fn intern_with_name(&self, name: String, text: &str, is_leaf: bool) -> &MPSegment {
        let is_var = name == constants::VAR_RULE_NAME;
        let in_var_range = name.starts_with(constants::VAR_RANGE_PREFIX);
        let unique = name.starts_with(constants::UNIQUE_PREFIX);

        let mut map = self.segments.borrow_mut();
        let key = self.calculate_hash(&name, text, is_leaf);

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
