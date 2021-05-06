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
import os
import sqlite3
from typing import Any, Dict, List, Union
import argparse
import time

DB_SCHEMA = """
CREATE TABLE [Facts]
(      [fact_id] INTEGER PRIMARY KEY AUTOINCREMENT,
       [subj] VARCHAR(255) NOT NULL,
       [verb] VARCHAR(255) NOT NULL,
       [objt] VARCHAR(255) NOT NULL
);
CREATE INDEX IF NOT EXISTS [factIx] ON [Facts] ([subj], [verb], [objt]);
CREATE INDEX IF NOT EXISTS [objIx] ON [Facts] ([objt]);
"""


INSERT = "INSERT INTO Facts (subj, verb, objt) VALUES (?, ?, ?);"
QUERY = "SELECT f1.objt FROM Facts as f1, Facts as f2 WHERE f1.subj = ? and f1.verb = ? and f1.objt = f2.objt and f2.subj = ? and f2.verb = ?;"

SETS = ["thing", "animal", "mammal", "primate", "human"]
NSETS = 5

parser = argparse.ArgumentParser(description='Benchmark on ont.')
parser.add_argument('-f', dest='f', type=int, default=2,
                    help='number of facts to add')
parser.add_argument('-r', dest='r', type=int, default=1,
                    help='report every r facts')


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

    def _db_execute(self, stmt: str, args: tuple = ()):
        self.db.execute(stmt, args)

    def _db_query(
        self, query: str, args: tuple = (), one: bool = False
    ) -> Union[List[Dict[str, Any]], Dict[str, Any], None]:
        cur = self.db.execute(query, args)
        rv = cur.fetchall()
        cur.close()
        return (rv[0] if rv else None) if one else rv

    def _db_commit(self):
        self.db.commit()

    def tell(self, subj, verb, objt):
        """
        """
        self._db_execute(INSERT, (subj, verb, objt))
        self._db_commit()

    def ask(self, subj1, verb1, subj2, verb2):
        """
        """
        return self._db_query(QUERY, (subj1, verb1, subj2, verb2))


if __name__ == '__main__':
    args = parser.parse_args()
    t0 = time.time()
    start = 0
    sqdb = SqliteBench()

    for i in range(args.f):

        start += 1
        s = SETS[i % NSETS]
        name = f"{s}{i}{start}"
        sqdb.tell(name, f"ISA{start}", s)

        if (i % args.r) == 0:
            t1 = time.time()
            start += 1
            sqdb.tell('johnny', f"ISA{start}", 'person')
            sqdb.tell('susan', f"ISA{start}", 'person')
            t2 = time.time()

            t_f = (t2 - t1) * 500000.0

            resp = sqdb.ask("johnny", f"ISA{start}", "susan", f"ISA{start}")
            if len(resp) == 0:
                print(f"Wrong resp for {name}")

            t3 = time.time()

            t_q = (t3 - t2) * 1000000.0

            print(f"  round {i}, duration: fact {t_f} usec, query {t_q} usec")

    t4 = time.time()
    t_t = (t4 - t0)

    print(f"total time: {t_t} sec for {start} entries")
