# -*- coding: utf-8 -*-
#
# Copyright (c) 2021 SUNET
# All rights reserved.
#
#   Redistribution and use in source and binary forms, with or
#   without modification, are permitted provided that the following
#   conditions are met:
#
#     1. Redistributions of source code must retain the above copyright
#        notice, this list of conditions and the following disclaimer.
#     2. Redistributions in binary form must reproduce the above
#        copyright notice, this list of conditions and the following
#        disclaimer in the documentation and/or other materials provided
#        with the distribution.
#     3. Neither the name of the SUNET nor the names of its
#        contributors may be used to endorse or promote products derived
#        from this software without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
# "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
# LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS
# FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE
# COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT,
# INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING,
# BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
# LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
# CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
# LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN
# ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
# POSSIBILITY OF SUCH DAMAGE.
#
import sqlite3
from typing import Any, Dict, List, Union
import argparse
import random
import time

DB_SCHEMA = """
CREATE TABLE [Leaf]
(      [leaf_id] INTEGER PRIMARY KEY AUTOINCREMENT,
       [name] INTEGER
);
CREATE TABLE [Branch]
(      [branch_id] INTEGER PRIMARY KEY AUTOINCREMENT,
       [name] INTEGER
);
CREATE TABLE [Child]
(
        [child_id] INTEGER PRIMARY KEY AUTOINCREMENT,
        [parent_branch] INTEGER,
        [idx] INTEGER,
        [is_branch] BOOLEAN,
        [branch] INTEGER,
        [leaf] INTEGER,
            FOREIGN KEY ([parent_branch]) REFERENCES [Branch] ([branch_id])
               ON DELETE NO ACTION ON UPDATE NO ACTION
            FOREIGN KEY ([branch]) REFERENCES [Branch] ([branch_id])
               ON DELETE NO ACTION ON UPDATE NO ACTION
            FOREIGN KEY ([leaf]) REFERENCES [Leaf] ([leaf_id])
               ON DELETE NO ACTION ON UPDATE NO ACTION

);
CREATE INDEX IF NOT EXISTS [leafNameIx] ON [Leaf] ([name]);
CREATE INDEX IF NOT EXISTS [branchNameIx] ON [Branch] ([name]);
CREATE INDEX IF NOT EXISTS [childBranchIx] ON [Child] ([parent_branch], [branch]);
CREATE INDEX IF NOT EXISTS [childLeafIx] ON [Child] ([parent_branch], [leaf]);
CREATE INDEX IF NOT EXISTS [childPBIx] ON [Child] ([parent_branch]);
CREATE INDEX IF NOT EXISTS [childBIx] ON [Child] ([branch]);
CREATE INDEX IF NOT EXISTS [childLIx] ON [Child] ([leaf]);
CREATE INDEX IF NOT EXISTS [childIDIx] ON [Child] ([idx]);
CREATE INDEX IF NOT EXISTS [childISIx] ON [Child] ([is_branch]);
CREATE INDEX IF NOT EXISTS [childISDIx] ON [Child] ([idx], [is_branch]);
"""


INSERT_LEAF = "INSERT INTO Leaf (name) VALUES (?);"
QUERY_LEAF = "SELECT leaf_id FROM Leaf WHERE name = ?;"
INSERT_BRANCH = "INSERT INTO Branch (name) VALUES (?);"
QUERY_BRANCH = "SELECT branch_id FROM Branch WHERE name = ?;"
INSERT_BCHILD = "INSERT INTO Child (parent_branch, idx, is_branch, branch) VALUES (?, ?, 1, ?)"
INSERT_LCHILD = "INSERT INTO Child (parent_branch, idx, is_branch, leaf) VALUES (?, ?, 0, ?)"
QUERY_BCHILD = "SELECT child_id FROM Child WHERE parent_branch = ? AND idx = ? AND is_branch = 1 AND branch = ?;"
QUERY_LCHILD = "SELECT child_id FROM Child WHERE parent_branch = ? AND idx = ? AND is_branch = 0 AND leaf = ?;"


parser = argparse.ArgumentParser(description='Benchmark on ont.')
parser.add_argument('-f', dest='f', type=int, default=2,
                    help='number of facts to add')
parser.add_argument('-r', dest='r', type=int, default=1,
                    help='report every r facts')
parser.add_argument('-d', dest='d', type=int, default=1,
                    help='max depth of tree')
parser.add_argument('-l', dest='l', type=int, default=1,
                    help='max length of branches')


def make_dicts(cursor, row):
    """
    See https://flask.palletsprojects.com/en/1.1.x/patterns/sqlite3
    """
    return dict((cursor.description[idx][0], value) for idx, value in enumerate(row))


def get_db(db_path):
    db = sqlite3.connect(db_path)

    db.cursor().executescript(DB_SCHEMA)
    db.commit()

    db.row_factory = make_dicts

    return db


def close_connection(exception):
    global db
    if db is not None:
        db.close()


class SqliteBench():
    """
    """
    def __init__(self, db_path=':memory:'):
        self.db_path = db_path
        self.db = get_db(db_path)

    def execute(self, stmt: str, args: tuple = ()):
        self.db.execute(stmt, args)

    def query(
        self, query: str, args: tuple = (), one: bool = False
    ) -> Union[List[Dict[str, Any]], Dict[str, Any], None]:
        cur = self.db.execute(query, args)
        rv = cur.fetchall()
        cur.close()
        return (rv[0] if rv else None) if one else rv

    def commit(self):
        self.db.commit()


def make_tree(db, depth, length):
    do_make_tree(db, depth, depth, length)
    db.commit()


def do_make_tree(db, depth, max_depth, length):
    if depth == 0 or random.randrange(8) > 5:
        leaf_name = random.randrange(10 ** (max_depth - depth + 1))
        db.execute(INSERT_LEAF, (leaf_name,))
        return db.query(QUERY_LEAF, (leaf_name,), one=True)['leaf_id'], False

    branch_name = random.randrange(10 ** (max_depth - depth + 1))
    db.execute(INSERT_BRANCH, (branch_name,))
    branch_id = db.query(QUERY_BRANCH, (branch_name,), one=True)['branch_id']

    for n in range(random.randrange(length)):

        new_id, is_branch = do_make_tree(db, depth - 1, max_depth, length)
        if is_branch:
            db.execute(INSERT_BCHILD, (branch_id, n, new_id))
        else:
            db.execute(INSERT_LCHILD, (branch_id, n, new_id))

    return branch_id, True


def make_tree_full(db, depth, length):
    do_make_tree_full(depth, depth, length)
    db.commit()


def do_make_tree_full(db, depth, max_depth, length):
    if depth == 0:
        leaf_name = random.randrange(10 ** (max_depth - depth + 1))
        db.execute(INSERT_LEAF, (leaf_name,))
        return db.query(QUERY_LEAF, (leaf_name,), one=True)['leaf_id'], False

    branch_name = random.randrange(10 ** (max_depth - depth + 1))
    db.execute(INSERT_BRANCH, (branch_name,))
    branch_id = db.query(QUERY_BRANCH, (branch_name,), one=True)['branch_id']

    for n in range(length):

        new_id, is_branch = do_make_tree_full(db, depth - 1, max_depth, length)
        if is_branch:
            db.execute(INSERT_BCHILD, (branch_id, n, new_id))
        else:
            db.execute(INSERT_LCHILD, (branch_id, n, new_id))

    return branch_id, True


# recursion_depth
rd = 0


def make_tree_q(db, depth, length):
    select = []
    fro = ["Branch as b1"]
    join = []
    where = []
    do_make_tree_q(db, depth, depth, length, select, join, where)
    db.commit()
    global rd
    rd = 0
    s = ', '.join(select)
    f = ', '.join(fro)
    w = ' AND '.join(where)
    j = ' '.join(join)
    q = f"SELECT DISTINCT {s} FROM {f} {j} WHERE {w};"
    return q


def do_make_tree_q(db, depth, max_depth, length, select, join, where):
    global rd
    rd += 1
    is_branch = False
    if depth == 0:
        leaf_name = random.randrange(100 ** (max_depth - depth + 1))
        leaf = db.query(QUERY_LEAF, (leaf_name,))
        if len(leaf) == 0:
            db.execute(INSERT_LEAF, (leaf_name,))
            new_id = db.query(QUERY_LEAF, (leaf_name,), one=True)['leaf_id']
        else:
            new_id = leaf[0]['leaf_id']

        child = f"l{rd}.leaf_id"
        select.append(f"{child} as l{rd}id")
        child_t = f"Leaf as l{rd}"
        where.append(f"l{rd}.name = {leaf_name}")

    else:
        branch_name = random.randrange(10 ** (max_depth - depth + 1))
        db.execute(INSERT_BRANCH, (branch_name,))
        new_id = db.query(QUERY_BRANCH, (branch_name,), one=True)['branch_id']

        where.append(f"b{rd}.name = {branch_name}")

        parent = child = f"b{rd}.branch_id"
        select.append(f"{child} as b{rd}id")
        child_t = f"Branch as b{rd}"

        is_branch = True

        for n in range(length):
            prejoin = []
            prewhere = []
            child_spec, child_table, child_id, child_is_branch = do_make_tree_q(db, depth - 1, max_depth, length, select, prejoin, prewhere)
            if child_is_branch:
                db.execute(INSERT_BCHILD, (new_id, n, child_id))
            else:
                db.execute(INSERT_LCHILD, (new_id, n, child_id))

            rd += 1
            join.append(f"JOIN Child as ch{rd} ON ch{rd}.parent_branch = {parent}")
            if child_is_branch:
                join.append(f"JOIN {child_table} ON {child_spec} = ch{rd}.branch")
            else:
                join.append(f"JOIN {child_table} ON {child_spec} = ch{rd}.leaf")

            join.extend(prejoin)

            where.append(f"ch{rd}.idx = {n}")
            where.append(f"ch{rd}.is_branch = {int(child_is_branch)}")
            where.extend(prewhere)

    return child, child_t, new_id, is_branch


if __name__ == '__main__':
    args = parser.parse_args()
    t0 = time.time()
    sqdb = SqliteBench()

    for i in range(args.f):

        make_tree(sqdb, args.d, args.l)

        if (i % args.r) == 0:
            t1 = time.time()

            q = make_tree_q(sqdb, args.d, args.l)

            t2 = time.time()

            t_f = (t2 - t1) * 1000000.0

            resp = sqdb.query(q, ())
            if len(resp) != 1:
                print(f"Wrong resp for {q}, expected 1, found {resp}")

            t3 = time.time()

            t_q = (t3 - t2) * 1000000.0

            print(f"{t_f:.3f}  {t_q:.3f}")

    t4 = time.time()
    t_t = (t4 - t0)

    print(f"total time: {t_t} sec for {args.f} entries")
