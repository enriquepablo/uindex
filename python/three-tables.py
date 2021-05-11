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
import time

DB_SCHEMA = """
CREATE TABLE [User]
(      [user_id] INTEGER PRIMARY KEY AUTOINCREMENT,
       [userid] VARCHAR(255) NOT NULL,
       [given_name] VARCHAR(255) NOT NULL,
       [surname] VARCHAR(255) NOT NULL
);
CREATE TABLE [City]
(      [city_id] INTEGER PRIMARY KEY AUTOINCREMENT,
       [name] VARCHAR(255) NOT NULL,
       [population] INTEGER,
       [country] VARCHAR(255) NOT NULL
);
CREATE TABLE [Address]
(      [address_id] INTEGER PRIMARY KEY AUTOINCREMENT,
       [user] INTEGER NOT NULL,
       [street] VARCHAR(255) NOT NULL,
       [number] INTEGER,
       [city] INTEGER NOT NULL,
            FOREIGN KEY ([user]) REFERENCES [User] ([user_id])
              ON DELETE NO ACTION ON UPDATE NO ACTION
            FOREIGN KEY ([city]) REFERENCES [City] ([city_id])
              ON DELETE NO ACTION ON UPDATE NO ACTION
);
CREATE INDEX IF NOT EXISTS [UidIx] ON [User] ([userid]);
CREATE INDEX IF NOT EXISTS [UgnIx] ON [User] ([given_name]);
CREATE INDEX IF NOT EXISTS [UsnIx] ON [User] ([surname]);
CREATE INDEX IF NOT EXISTS [UgsIx] ON [User] ([given_name], [surname]);
CREATE INDEX IF NOT EXISTS [CnmIx] ON [City] ([name]);
CREATE INDEX IF NOT EXISTS [CppIx] ON [City] ([population]);
CREATE INDEX IF NOT EXISTS [CctIx] ON [City] ([country]);
CREATE INDEX IF NOT EXISTS [AusIx] ON [Address] ([user]);
CREATE INDEX IF NOT EXISTS [AstIx] ON [Address] ([street]);
CREATE INDEX IF NOT EXISTS [AnmIx] ON [Address] ([number]);
CREATE INDEX IF NOT EXISTS [ActIx] ON [Address] ([city]);
"""


INSERT_USER = "INSERT INTO User (userid, given_name, surname) VALUES (?, ?, ?);"
INSERT_CITY = "INSERT INTO City (name, population, country) VALUES (?, ?, ?);"
INSERT_ADDRESS = "INSERT INTO Address (user, street, number, city) VALUES (?, ?, ?, ?);"

QUERY_USER = "SELECT user_id FROM User WHERE userid = ? AND given_name = ? AND surname = ?;"
QUERY_CITY = "SELECT city_id FROM City WHERE name = ? AND population = ? AND country = ?;"
QUERY_CITY_BY_NAME = "SELECT city_id FROM City WHERE name = ?;"

QUERY = "SELECT c.population, c.country FROM User as u JOIN Address as a ON a.user = u.user_id JOIN City as c ON c.city_id = a.city WHERE u.given_name = ? AND u.surname = ?;"


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

    def tell_user(self, userid, given_name, surname):
        """
        """
        self._db_execute(INSERT_USER, (userid, given_name, surname))
        self._db_commit()

    def tell_city(self, name, population, country):
        """
        """
        self._db_execute(INSERT_CITY, (name, population, country))
        self._db_commit()

    def tell_address(self, user, street, number, city):
        """
        """
        self._db_execute(INSERT_ADDRESS, (user, street, number, city))
        self._db_commit()

    def query_user(self, userid, given_name, surname):
        """
        """
        return self._db_query(QUERY_USER, (userid, given_name, surname), one=True)['user_id']

    def query_city(self, name, population, country):
        """
        """
        return self._db_query(QUERY_CITY, (name, population, country), one=True)['city_id']

    def query_city_by_name(self, name):
        """
        """
        return self._db_query(QUERY_CITY_BY_NAME, (name,), one=True)['city_id']

    def query(self, given_name, surname):
        """
        """
        return self._db_query(QUERY, (given_name, surname), one=True)


if __name__ == '__main__':
    args = parser.parse_args()
    t0 = time.time()
    start = 0
    db = SqliteBench()

    for i in range(args.f):

        start += 1

        userid = f"user{start}"
        given_name = f"John{start}"
        surname = f"Smith{start}"

        city = f"city{start % 100}"

        t1 = time.time()

        db.tell_user(userid, given_name, surname)
        user_id = db.query_user(userid, given_name, surname)

        if i < 100:
            population = f"{start * 1000}"
            country = f"country{start % 50}"

            try:
                city_id = db.query_city(city, population, country)
            except Exception:
                db.tell_city(city, population, country)
                city_id = db.query_city(city, population, country)
        else:
            city_id = db.query_city_by_name(city)

        street = f"Lane{start % 1000}"
        number = f"{start}"
        city = f"city{start % 100}"

        db.tell_address(user_id, street, number, city_id)

        t2 = time.time()

        if (i % args.r) == 0:
            t3 = time.time()

            resp = db.query(given_name, surname)
            if len(resp) == 0:
                print(f"Wrong resp for {surname}")

            t4 = time.time()

            t_f = (t2 - t1) * 1000000.0
            t_q = (t4 - t3) * 1000000.0

            print(f"{t_f:.3f}  {t_q:.3f}")

    t5 = time.time()
    t_t = (t5 - t0)

    print(f"total time: {t_t} sec for {start} entries")
