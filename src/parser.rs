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
//

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

use proc_macro2::TokenStream;


pub fn derive_parser(attr: &syn::Attribute) -> TokenStream {
    quote! {

        pub struct StringCache(RefCell<HashSet<String>>);

        impl StringCache {
            fn new() -> Self {
                StringCache(RefCell::new(HashSet::new()))
            }

            fn intern<'a>(&'a self, s: &str) -> &'a str {
                let mut set = self.0.borrow_mut();
                let mut interned = set.get(s);

                if interned.is_none() {
                    set.insert(s.into());
                    interned = set.get(s);
                }
                unsafe { mem::transmute(interned.unwrap().as_str()) }
            }
        }

        pub struct MPParser {
            pub lexicon: Box<Lexicon>,
        }

        #[derive(Parser)]
        #attr
        pub struct FactParser;

        impl<'a> MPParser {

            pub fn new() -> MPParser {
                MPParser {
                    lexicon: Box::new(Lexicon::new()),
                }
            }

            pub fn parse_text(&'a self, text: &'a str) -> Result<ParseResult<'a>, Error<kparser::Rule>> {
                let parse_tree = kparser::KParser::parse(kparser::Rule::knowledge, text)?.next().expect("initial parse tree");
                let mut facts: Vec<&'a str> = vec![];
                for pair in parse_tree.into_inner() {
                    match pair.as_rule() {
                        kparser::Rule::fact => {
                            facts.push(pair.as_str());
                        },
                        _ => {}
                    }
                }
                Ok(ParseResult { facts })
            }

            pub fn parse_fact(&'a self, text: &'a str) -> Vec<MPPath<'a>> {
                let parse_tree = FactParser::parse(Rule::fact, text).ok().expect("fact pairset").next().expect("fact pair");
                self.visit_parse_node(parse_tree,
                                      vec![],
                                      vec![],
                                      0)
            }

            fn visit_parse_node(&'a self,
                                parse_tree: Pair<'a, Rule>,
                                mut root_segments: Vec<&'a MPSegment>,
                                mut all_paths: Vec<MPPath<'a>>,
                                index: usize,
                            ) -> Vec<MPPath> {
                let text = parse_tree.as_str();
                if text.is_empty() {
                    return all_paths;
                }
                let rule = parse_tree.as_rule();
                let name = format!("{:?}", rule);
                let can_be_var = name.starts_with(constants::VAR_RANGE_PREFIX);
                let mut children = parse_tree.into_inner().peekable();
                let is_leaf = children.peek().is_none();
                let segment = self.lexicon.intern_with_name(name, text, is_leaf);
                root_segments.push(segment);
                let root_ref: &Vec<&MPSegment> = unsafe { mem::transmute( &root_segments ) };
                if can_be_var || is_leaf {
                    let segments = root_segments;
                    let new_path = MPPath::new(segments);
                    all_paths.push(new_path);
                }
                let mut new_index = 0;
                for child in children {
                    all_paths = self.visit_parse_node(child,
                                                      root_ref.clone(),
                                                      all_paths,
                                                      new_index);
                    new_index += 1;
                }
                all_paths
            }
        }
    }
}
