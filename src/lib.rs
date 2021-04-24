// Copyright (c) 2020 by Enrique Pérez Arnaud <enrique at cazalla.net>    
//    
// This file is part of the uindex project.    
// http://www.uindex.net    
//    
// The uindex project is free software: you can redistribute it and/or modify    
// it under the terms of the GNU General Public License as published by    
// the Free Software Foundation, either version 3 of the License, or    
// (at your option) any later version.    
//    
// The uindex project is distributed in the hope that it will be useful,    
// but WITHOUT ANY WARRANTY; without even the implied warranty of    
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the    
// GNU General Public License for more details.    
//    
// You should have received a copy of the GNU General Public License    
// along with any part of the uindex project.    
// If not, see <http://www.gnu.org/licenses/>.
//!
//! uindex generates inference engines on top of Parsing Expression Grammars (PEGs).
//! It allows you to specify the syntax of your facts in a PEG, and
//! automatically obtain an inference engine able to deal with knowledge bases
//! composed of facts and rules compliant with the provided PEG.
//!
//! So on one hand, uindex helps dealing with data, in whatever form or shape.
//! It allows you to keep your data in knowledge bases under any shape
//! and structural detail you may feel appropriate,
//! and to query and massage it efficiently
//! at any level of the detail you may have bothered to specify.
//! 
//! On the other hand, uindex allows you to develop programs
//! under the paradigm of logic programming,
//! with a syntax that is exactly as expressive and clear as you care to specify.
//!
//! Finally, it is worth noting that uindex is very performant,
//! and furthermore, that its performance is fully independent of the size of the knowledge bases
//! it deals with.
//!
//! Check out the [README](https://gitlab.com/enriquepablo/uindex) for more detailed info.
//!

#![feature(hash_set_entry)]
#![allow(dead_code)]


pub mod constants;
pub mod segment;
pub mod matching;
pub mod path;
//pub mod fact;
mod parser;
pub mod facttree;
mod knowledge;
pub mod lexicon;
pub mod kbase;
pub mod kparser;
pub mod parse_result;


extern crate pest;
#[macro_use]
extern crate pest_derive;

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro2::TokenStream;


pub fn derive_dbase(input: proc_macro::TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = ast.ident;
    let attr = &ast.attrs[0];

    let derived_parser = parser::derive_parser(attr);
    let derived_db = knowledge::derive_db();

    quote! {

        use std::collections::{ HashMap, HashSet, VecDeque };
        use std::cell::RefCell;
        use std::mem;

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        use log::{info, trace};

        use pest::error::Error;
        use pest::Parser;
        use pest::iterators::Pair;
        use uindex::constants;
        use uindex::facttree::FactSet;
        use uindex::kbase::{ DataBase, DBGen };
        use uindex::lexicon::Lexicon;
        use uindex::matching::{ MPMatching };
        use uindex::path::MPPath;
        use uindex::segment::MPSegment;
        use uindex::kparser;
        use uindex::parse_result::ParseResult;


        #derived_parser

        #derived_db

        impl<'a> DBGen<'a> for #name {
            type Output = DB<'a>;
            fn gen_db() -> DB<'a> {
                DB::new()
            }
        }
    }
}
