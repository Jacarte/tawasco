import sys
import os
import subprocess
import json
import zipfile as zzip
import shutil
from peewee import Model, BooleanField, TextField, ForeignKeyField, IntegerField, SqliteDatabase
from blake3 import blake3

NAME = sys.argv[1]
NAME = NAME.split(".")[0]
db = SqliteDatabase(f'{NAME}.c.db')

class BaseModel(Model):
    class Meta:
        database = db

class Wasm(BaseModel):
    parent = ForeignKeyField('self', null=True)
    original = ForeignKeyField('self', null=True)
    optimized = BooleanField()
    hash = TextField()
    name = TextField()
    parent_name = TextField(null=True)

class MutationInfo(BaseModel):
    wasm = ForeignKeyField(Wasm)
    description = TextField(null=True)

class OptimizationInfo(BaseModel):
    wasm = ForeignKeyField(Wasm)
    description = TextField()
    stdout = TextField(null=True)
    stderr = TextField(null=True)

class Cwasm(BaseModel):
    wasm = ForeignKeyField(Wasm)
    level = IntegerField()
    hash = TextField()
    stdout = TextField(null=True)
    stderr = TextField(null=True)


class V8(BaseModel):
    wasm = ForeignKeyField(Wasm)
    level = IntegerField()  # Needs validation to be 0 or 1
    hash = TextField()
    stdout = TextField(null=True)
    stderr = TextField(null=True)

class Wat(BaseModel):
    wasm = ForeignKeyField(Wasm)
    flags = TextField()
    hash = TextField()
    stdout = TextField(null=True)
    stderr = TextField(null=True)


def init():
    db.connect()
    db.create_tables([Wasm, MutationInfo, OptimizationInfo, Cwasm, V8, Wat])

def connect():
    db.connect()

def max_sql_variables():
    """Get the maximum number of arguments allowed in a query by the current
    sqlite3 implementation. Based on `this question
    `_

    Returns
    -------
    int
        inferred SQLITE_MAX_VARIABLE_NUMBER
    """
    import sqlite3
    db = sqlite3.connect(':memory:')
    cur = db.cursor()
    cur.execute('CREATE TABLE t (test)')
    low, high = 0, 100000
    while (high - 1) > low: 
        guess = (high + low) // 2
        query = 'INSERT INTO t VALUES ' + ','.join(['(?)' for _ in
                                                    range(guess)])
        args = [str(i) for i in range(guess)]
        try:
            cur.execute(query, args)
        except sqlite3.OperationalError as e:
            if "too many SQL variables" in str(e):
                high = guess
            else:
                raise
        else:
            low = guess
    cur.close()
    db.close()
    return low

SQLITE_MAX_VARIABLE_NUMBER = max_sql_variables()
print("SQL MAX VARIABLES", SQLITE_MAX_VARIABLE_NUMBER)