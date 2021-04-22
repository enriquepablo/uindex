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

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

use proc_macro2::TokenStream;


pub fn derive_db() -> TokenStream {
    quote! {

        pub struct DB<'a> {
            mpparser: &'a MPParser<'a>,
            facts: FactSet<'a>,
        }
        impl<'a> DataBase<'a> for DB<'a> {
            fn tell(&'a self, knowledge: &'a str) {
                let result = self.mpparser.parse_text(knowledge.trim());
                if result.is_err() {
                    panic!("Parsing problem! {}", result.err().unwrap());
                } else {
                    let ParseResult { facts } = result.ok().unwrap();
                    for fact in facts {

                        let fact_paths = self.mpparser.parse_fact(fact);
                        let (exists, paths) = self.facts.ask_fact_bool(fact_paths);
                        if  exists {
                            return;
                        }
                        self.facts.add_fact(paths);
                    }
                }
            }
            fn ask(&'a self, knowledge: &'a str) -> Vec<MPMatching<'a>> {
                let ParseResult { mut facts, .. } = self.mpparser.parse_text(knowledge).ok().expect("parse result");
                let fact = facts.pop().unwrap();
                let q = self.mpparser.parse_fact(fact);
                let (resp, _, _) = self.facts.ask_fact(q);
                resp
            }
        }
        impl<'a> DB<'a> {

            pub fn new () -> DB<'a> {
                Self {
                    mpparser: Box::leak(Box::new(MPParser::new())),
                    facts: FactSet::new(),
                }
            }
        }
    }
}
