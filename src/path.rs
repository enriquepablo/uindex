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

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::fmt;

use crate::segment::MPSegment;
use crate::matching::{ MPMatching, get_or_key };


#[derive(Debug, Clone)]
pub struct TSegment {
    pub name: u64,
    pub text: u64,
}

impl fmt::Display for TSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.text)
    }
}

impl PartialEq for TSegment {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.text == other.text
    }
}

impl Eq for TSegment {}

impl Hash for TSegment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.text.hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct MPPath<'a> {
    pub value: &'a MPSegment,
    pub segments: Vec<TSegment>,
    identity: u64,
}

impl<'a> MPPath<'a> {
    pub fn new(segments: Vec<TSegment>, value: &'a MPSegment) -> MPPath {
        let mut hasher = DefaultHasher::new();
        for segment in segments.iter() {
            segment.name.hash(&mut hasher);
        }
        value.name.hash(&mut hasher);
        value.text.hash(&mut hasher);
        let identity = hasher.finish();
        MPPath { value, segments, identity }
    }
    pub fn len(&self) -> usize {
        self.segments.len()
    }
    pub fn starts_with(&self, path: &MPPath) -> bool {
        let lpath = path.len();
        self.len() >= lpath && &self.segments[0..lpath] == &path.segments[0..lpath]
    }
    pub fn paths_after(&'a self, paths: &'a [MPPath]) -> usize {
        let mut seen = false;
        let mut path_starts_with_self: bool;
        let mut i = 0;
        for path in paths {
            if path.value.is_empty {
                i += 1;
                continue;
            }
            path_starts_with_self = path.starts_with(&self);
            if path_starts_with_self {
                seen = true;
            } else if seen {
                break;
            }
            i += 1;
        }
        i as usize
    }

    pub fn substitute(&'a self, matching: &'a MPMatching) -> MPPath {
        let new_segments = self.segments.clone();
        let new_value = get_or_key(&matching, &self.value);
        MPPath::new(new_segments, new_value)
    }
}


impl<'a> fmt::Display for MPPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.value)
    }
}

impl<'a> PartialEq for MPPath<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl<'a> Eq for MPPath<'_> {}

impl<'a> Hash for MPPath<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identity.hash(state);
    }
}
