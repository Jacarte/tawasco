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
