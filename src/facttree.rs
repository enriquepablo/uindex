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

use std::cell::RefCell;
use std::clone::Clone;
use std::collections::HashMap;
use std::lazy::OnceCell;
use std::mem;

use crate::matching::MPMatching;
use crate::path::MPPath;
use crate::segment::MPSegment;

pub struct CarryOver<'a>(HashMap<usize, &'a FSNode<'a>>);

impl<'a> CarryOver<'a> {
    pub fn add(mut self, index: usize, node: &'a FSNode<'a>) -> Self {
        self.0.insert(index, node);
        self
    }
    pub fn node(mut self, index: usize) -> (Self, Option<&'a FSNode<'a>>) {
        let node_opt = self.0.remove(&index);
        (self, node_opt)
    }
}

fn mk_children<'a>() -> RefCell<HashMap<u64, &'a FSNode<'a>>> {
    RefCell::new(HashMap::with_capacity(0))
}

pub struct FactSet<'a> {
    pub root: Box<FSNode<'a>>,
}

impl<'a> FactSet<'a> {
    pub fn new() -> FactSet<'a> {
        FactSet {
            root: Box::new(FSNode::new(None)),
        }
    }
    pub fn add_fact(&'a self, fact: Vec<MPPath<'a>>) {
        let carry = CarryOver(HashMap::new());
        self.follow_and_create_paths(&self.root, fact, carry);
    }
    pub fn ask_fact(&'a self, fact: Vec<MPPath<'a>>) -> (Vec<MPMatching<'a>>, Vec<MPPath<'a>>) {
        let response: Vec<MPMatching> = vec![];
        let matching: MPMatching = HashMap::new();
        let paths: &[MPPath] = unsafe { mem::transmute(fact.as_slice()) };
        let npaths = vec![paths];
        let qpaths: &[&[MPPath]] = unsafe { mem::transmute(npaths.as_slice()) };
        let response = self
            .root
            .query_paths(qpaths, matching, response, Some(&(*self.root)));
        (response, fact)
    }
    pub fn ask_fact_bool(&'a self, fact: Vec<MPPath<'a>>) -> (bool, Vec<MPPath<'a>>) {
        let (resp, fact) = self.ask_fact(fact);
        (resp.len() > 0, fact)
    }
    pub fn ask_facts(&'a self, facts: Vec<Vec<MPPath<'a>>>) -> Vec<MPMatching<'a>> {
        let response: Vec<MPMatching> = vec![];
        let matching: MPMatching = HashMap::new();
        let paths: Vec<&[MPPath]> = facts.iter().map(|fact| fact.as_slice()).collect();
        let qpaths: &[&[MPPath]] = unsafe { mem::transmute(paths.as_slice()) };
        let response = self
            .root
            .query_paths(qpaths, matching, response, Some(&(*self.root)));
        response
    }
    pub fn follow_and_create_paths(
        &'a self,
        mut parent: &'a FSNode<'a>,
        mut paths: Vec<MPPath<'a>>,
        mut carry: CarryOver<'a>,
    ) {
        let mut child: &FSNode;
        let mut path_index = 0;
        while paths.len() > 0 {
            let path = paths.remove(0);
            if path.value.is_empty {
                path_index += 1;
                continue;
            }
            let opt_child = parent.get_lchild(path.identity);
            let reindex = path.paths_after(&paths);
            if opt_child.is_some() {
                child = opt_child.expect("node");
                if !path.value.is_leaf {
                    carry = carry.add(reindex, child);
                    path_index += 1;
                    continue;
                }
            } else if path.value.is_leaf {
                paths.insert(0, path);
                self.create_paths(parent, paths, carry, path_index);
                return;
            } else {
                let unique_child = path.value.unique;
                let path_id = path.identity;
                let child_node = FSNode::new(Some(path.value));
                let (new_child, new_carry) = self.intern_lchild(
                    parent,
                    path_id,
                    unique_child,
                    child_node,
                    carry,
                    path_index,
                );
                child = new_child;
                carry = new_carry;

                carry = carry.add(reindex, child);
                path_index += 1;
                continue;
            }
            parent = child;
            path_index += 1;
        }
    }

    fn create_paths(
        &'a self,
        mut parent: &'a FSNode<'a>,
        mut paths: Vec<MPPath<'a>>,
        mut carry: CarryOver<'a>,
        offset: usize,
    ) {
        let mut child: &FSNode;
        let mut path_index = 0;
        while paths.len() > 0 {
            let path = paths.remove(0);
            if path.value.is_empty {
                path_index += 1;
                continue;
            }
            let path_id = path.identity;
            let unique_child = path.value.unique;
            let is_leaf = path.value.is_leaf;
            let reindex = path.paths_after(&paths);
            let child_node = FSNode::new(Some(path.value));
            let real_index = path_index + offset;
            let (new_child, new_carry) = self.intern_lchild(
                parent,
                path_id,
                unique_child,
                child_node,
                carry,
                real_index,
            );
            child = new_child;
            carry = new_carry;
            if !is_leaf {
                carry = carry.add(reindex, child);
                path_index += 1;
                continue;
            }
            parent = child;
            path_index += 1;
        }
    }
    pub fn intern_lchild(
        &'a self,
        parent: &'a FSNode<'a>,
        path_id: u64,
        unique_child: bool,
        child: FSNode<'a>,
        mut carry: CarryOver<'a>,
        index: usize,
    ) -> (&'a FSNode<'a>, CarryOver<'a>) {
        let child_ref = Box::leak(Box::new(child));
        let (new_carry, more) = carry.node(index);
        carry = new_carry;
        if more.is_some() {
            let mut other_parent = more
                .unwrap()
                .lchildren
                .get_or_init(mk_children)
                .borrow_mut();
            other_parent.insert(path_id, child_ref);
        }
        let mut one_parent = parent.lchildren.get_or_init(mk_children).borrow_mut();
        if unique_child {
            one_parent.clear();
        }
        one_parent.insert(path_id, child_ref);
        (child_ref, carry)
    }
}

#[derive(Debug)]
pub struct FSNode<'a> {
    lchildren: OnceCell<RefCell<HashMap<u64, &'a FSNode<'a>>>>,
    value: Option<&'a MPSegment>,
}

impl<'a> FSNode<'a> {
    pub fn new(value: Option<&'a MPSegment>) -> FSNode<'a> {
        FSNode {
            lchildren: OnceCell::new(),
            value,
        }
    }
    pub fn get_lchild(&'a self, path_id: u64) -> Option<&'a Self> {
        let children = self.lchildren.get();
        if children.is_none() {
            return None;
        }
        let ch = children.unwrap().borrow();
        match ch.get(&path_id) {
            None => None,
            Some(child_ref) => Some(*child_ref),
        }
    }
    pub fn get_lchild_r(&'a self, path: &'a MPPath<'a>) -> Option<&'a Self> {
        let children = self.lchildren.get();
        if children.is_none() {
            return None;
        }
        let ch = children.unwrap().borrow();
        match ch.get(&path.identity) {
            None => None,
            Some(child_ref) => Some(*child_ref),
        }
    }
    pub fn query_paths(
        &'a self,
        all_all_paths: &'a [&'a [MPPath]],
        matching: MPMatching<'a>,
        mut resp: Vec<MPMatching<'a>>,
        root: Option<&'a FSNode<'a>>,
    ) -> Vec<MPMatching<'a>> {
        let rroot: &'a FSNode;
        if root.is_some() {
            rroot = root.unwrap();
        } else {
            rroot = self;
        }
        let (&new_all_paths, new_all_all) = all_all_paths.split_first().unwrap();
        let mut all_paths = new_all_paths;

        let mut finished = false;
        let mut next_path: Option<&MPPath> = None;
        let mut next_paths: Option<&'a [MPPath]> = None;
        while !finished {
            let split_paths = all_paths.split_first();
            if split_paths.is_some() {
                let (path, paths) = split_paths.unwrap();
                if !path.value.is_empty && path.value.is_leaf {
                    finished = true;
                    next_path = Some(path);
                    next_paths = Some(paths);
                } else {
                    all_paths = paths;
                }
            } else {
                finished = true;
            }
        }
        if next_path.is_some() {
            let mut subs_path: Option<&MPPath> = None;
            let path = next_path.unwrap();
            let paths = next_paths.unwrap();
            if path.value.is_var {
                if !matching.contains_key(&path.value) {
                    let lchildren = self.lchildren.get();
                    if lchildren.is_some() {
                        // If there is a variable in the question and this is its 1st ocurrence,
                        // recurse over all the logical children in the present node
                        for lchild_node in lchildren.unwrap().borrow().values() {
                            let mut new_matching = matching.clone();
                            new_matching.insert(path.value, lchild_node.value.unwrap());
                            let mut npaths = vec![paths];
                            npaths.extend_from_slice(new_all_all);
                            let qpaths: &[&[MPPath]] = unsafe { mem::transmute(npaths.as_slice()) };
                            resp = lchild_node.query_paths(qpaths, new_matching, resp, Some(rroot));
                        }
                    }
                    return resp;
                } else {
                    // If there is a variable in the question and this is not its 1st ocurrence,
                    // recover the matched value and change the matching path accordingly,
                    // to be treated as non variable path
                    let matching_ref: &MPMatching = unsafe { mem::transmute(&matching) };
                    let new_path = path.substitute(matching_ref);
                    let new_path_ref = unsafe { mem::transmute(&new_path) };
                    subs_path = Some(new_path_ref);
                }
            }
            let next: Option<&FSNode>;
            let new_path: &MPPath;
            if subs_path.is_some() {
                new_path = subs_path.unwrap();
            } else {
                new_path = path;
            }
            next = self.get_lchild_r(new_path);
            if next.is_some() {
                let next_node = next.unwrap();
                let mut npaths = vec![paths];
                npaths.extend_from_slice(new_all_all);
                let qpaths: &[&[MPPath]] = unsafe { mem::transmute(npaths.as_slice()) };
                resp = next_node.query_paths(qpaths, matching, resp, Some(rroot));
            }
        } else {
            if new_all_all.len() > 0 {
                resp = rroot.query_paths(new_all_all, matching, resp, None);
            } else {
                resp.push(matching);
            }
        }
        resp
    }
}
