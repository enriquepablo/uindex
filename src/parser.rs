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
            fn calculate_name_hash(&self, name: &str) -> u64 {
                let mut s = DefaultHasher::new();
                name.hash(&mut s);
                s.finish()
            }
            fn calculate_hash(&self, name: &str, text: &str, is_leaf: bool) -> u64 {
                let mut s = DefaultHasher::new();
                name.hash(&mut s);
                text.hash(&mut s);
                is_leaf.hash(&mut s);
                s.finish()
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
                                mut root_segments: Vec<TSegment>,
                                mut all_paths: Vec<MPPath<'a>>,
                                index: usize,
                            ) -> Vec<MPPath> {
                let text = parse_tree.as_str();
                if text.is_empty() {
                    return all_paths;
                }
                let rule = parse_tree.as_rule();
                let name = format!("{:?}", rule);
                let is_var = name == constants::VAR_RULE_NAME;
                let in_var_range = name.starts_with(constants::VAR_RANGE_PREFIX);
                let unique = name.starts_with(constants::UNIQUE_PREFIX);
                let mut children = parse_tree.into_inner().peekable();
                let is_leaf = children.peek().is_none();
                let mut new_root_segments: Option<Vec<TSegment>> = None;
                if !is_leaf {
                    let mut pre_new_root_segments = root_segments.clone();
                    let text_hash = self.calculate_name_hash(text);
                    let name_hash = self.calculate_name_hash(name.as_str());
                    let tsegment = TSegment { name: name_hash, text: text_hash };
                    pre_new_root_segments.push(tsegment);
                    new_root_segments = Some(pre_new_root_segments);
                }
                if in_var_range || is_leaf {
                    let key = self.calculate_hash(name.as_str(), text, is_leaf);
                    let segment = self.lexicon.intern_with_name(self.calculate_name_hash(name.as_str()), text, key, is_leaf, is_var, in_var_range, unique);
                    let new_path = MPPath::new(root_segments, segment);
                    all_paths.push(new_path);
                }
                let mut new_index = 0;
                if !is_leaf {
                    let next_root_segments = new_root_segments.unwrap();
                    for child in children {
                        all_paths = self.visit_parse_node(child,
                                                          next_root_segments.clone(),
                                                          all_paths,
                                                          new_index);
                        new_index += 1;
                    }
                }
                all_paths
            }
        }
    }
}
